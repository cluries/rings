use crate::erx;

use base64::Engine;
use rand::Rng;

use block_padding::{Padding, Pkcs7};
use ofb::cipher::StreamCipher;

use aes::{
    cipher::{
        generic_array::{typenum, GenericArray},
        BlockDecrypt, BlockDecryptMut, BlockEncrypt, BlockEncryptMut, KeyInit, KeyIvInit,
    },
    Aes128,
};

use rsa::{
    pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey, EncodeRsaPrivateKey, EncodeRsaPublicKey},
    Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey,
};

/// Rand 库升级到0.9过后，和rasa库的rand_core::RngCore不兼容，需要适配一下
/// implement rsa::rand_core::RngCore for CompatRng
/// implement rsa::rand_core::CryptoRng for CompatRng
pub struct CompatRng<T: Rng> {
    inner: T,
}

impl<T: Rng> CompatRng<T> {
    pub fn thread_rng() -> CompatRng<rand::rngs::ThreadRng> {
        CompatRng { inner: rand::rngs::ThreadRng::default() }
    }
}

impl<T: Rng> rsa::rand_core::CryptoRng for CompatRng<T> {}

impl<T: Rng> rsa::rand_core::RngCore for CompatRng<T> {
    fn next_u32(&mut self) -> u32 {
        self.inner.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.inner.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.inner.fill_bytes(dest);
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rsa::rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

/// 表示AES加密算法的不同模式
#[derive(Debug)]
pub enum AESMode {
    /// ECB（Electronic Codebook，电子密码本模式）
    /// - 每个块独立加密，相同的明文块会生成相同的密文块。
    /// - 不需要IV或Nonce。
    ECB,

    /// CBC（Cipher Block Chaining，密码分组链接模式）
    /// - 每个明文块在加密前与前一个密文块进行异或操作。
    /// - 需要一个初始化向量（IV）来加密第一个块。
    CBC { iv: Vec<u8> },

    /// CFB（Cipher Feedback，密码反馈模式）
    /// - 将加密算法的输出反馈到输入中。
    /// - 需要一个初始化向量（IV）来启动加密过程。
    CFB { iv: Vec<u8> },

    /// OFB（Output Feedback，输出反馈模式）
    /// - 类似于CFB，但反馈的是加密算法的输出，而不是密文。
    /// - 需要一个初始化向量（IV）来生成密钥流。
    OFB { iv: Vec<u8> },

    /// CTR（Counter，计数器模式）
    /// - 使用一个计数器作为输入，每个块使用不同的计数器值。
    /// - 需要一个Nonce和计数器，通常将它们组合成一个IV。
    CTR { iv: Vec<u8> },

    /// GCM（Galois/Counter Mode，伽罗瓦计数器模式）
    /// - 提供加密和认证功能。
    /// - 需要一个Nonce（通常称为IV）和认证标签。
    GCM { iv: Vec<u8>, auth_tag: Option<Vec<u8>> },

    /// CCM（Counter with CBC-MAC，带CBC-MAC的计数器模式）
    /// - 提供加密和认证功能。
    /// - 需要一个Nonce（通常称为IV）。
    CCM { iv: Vec<u8> },
}

type GAB128 = GenericArray<u8, typenum::U16>;

impl AESMode {
    const BIT128_BLOCK_SIZE: usize = 16;

    /// 获取模式的名称
    pub fn name(&self) -> &'static str {
        match self {
            AESMode::ECB => "ECB",
            AESMode::CBC { .. } => "CBC",
            AESMode::CFB { .. } => "CFB",
            AESMode::OFB { .. } => "OFB",
            AESMode::CTR { .. } => "CTR",
            AESMode::GCM { .. } => "GCM",
            AESMode::CCM { .. } => "CCM",
        }
    }

    pub fn encrypt(&self, key: &[u8], payload: &[u8]) -> Result<Vec<u8>, erx::Erx> {
        match self {
            AESMode::ECB => self.ecb_encrypt(key, payload),
            AESMode::CBC { .. } => self.cbc_encrypt(key, payload),
            AESMode::CFB { .. } => self.cfb_encrypt(key, payload),
            AESMode::OFB { .. } => self.ofb_encrypt(key, payload),
            AESMode::CTR { .. } => self.ctr_encrypt(key, payload),
            AESMode::GCM { .. } => Err(erx::Erx::new("!unimplemented!")),
            AESMode::CCM { .. } => Err(erx::Erx::new("!unimplemented!")),
        }
    }

    pub fn generate_iv() -> Vec<u8> {
        let mut rng = rand::rng();
        let mut iv = vec![0u8; AESMode::BIT128_BLOCK_SIZE];
        rng.fill(&mut iv[..]);
        iv
    }

    pub fn decrypt(&self, key: &[u8], payload: &[u8]) -> Result<Vec<u8>, erx::Erx> {
        match self {
            AESMode::ECB => self.ecb_decrypt(key, payload),
            AESMode::CBC { .. } => self.cbc_decrypt(key, payload),
            AESMode::CFB { .. } => self.cfb_decrypt(key, payload),
            AESMode::OFB { .. } => self.ofb_decrypt(key, payload),
            AESMode::CTR { .. } => self.ctr_decrypt(key, payload),
            AESMode::GCM { .. } => Err(erx::Erx::new("!unimplemented!")),
            AESMode::CCM { .. } => Err(erx::Erx::new("!unimplemented!")),
        }
    }

    fn padding_buffer(payload: &[u8]) -> Vec<u8> {
        let mut payload_vec = payload.to_vec();
        let padding_count = AESMode::BIT128_BLOCK_SIZE - payload.len() % AESMode::BIT128_BLOCK_SIZE;
        payload_vec.extend(std::iter::repeat(0).take(padding_count));
        payload_vec
    }

    fn ecb_encrypt(&self, key: &[u8], payload: &[u8]) -> Result<Vec<u8>, erx::Erx> {
        let key = GAB128::from_slice(key);
        let cipher = Aes128::new(&key);

        let payload_length = payload.len();
        let block_count = payload_length / AESMode::BIT128_BLOCK_SIZE + 1;
        let mut result = Vec::with_capacity(block_count * AESMode::BIT128_BLOCK_SIZE);

        for i in 0..block_count {
            let mut block: GAB128 = GAB128::default();
            if (i + 1) * AESMode::BIT128_BLOCK_SIZE > payload_length {
                let tail_size = payload_length % AESMode::BIT128_BLOCK_SIZE;
                if tail_size > 0 {
                    block[..tail_size].clone_from_slice(&payload[i * AESMode::BIT128_BLOCK_SIZE..payload_length]);
                }
                Pkcs7::pad(&mut block, tail_size);
            } else {
                block.clone_from_slice(&payload[i * AESMode::BIT128_BLOCK_SIZE..(i + 1) * AESMode::BIT128_BLOCK_SIZE]);
            }

            cipher.encrypt_block(&mut block);
            result.extend_from_slice(&block);
        }

        Ok(result)
    }

    fn ecb_decrypt(&self, key: &[u8], payload: &[u8]) -> Result<Vec<u8>, erx::Erx> {
        if payload.len() % AESMode::BIT128_BLOCK_SIZE != 0 {
            return Err(erx::Erx::new("invalid payload size, must be a multiple of 16"));
        }

        let key = GAB128::from_slice(key);
        let cipher = Aes128::new(key);
        let block_count = payload.len() / AESMode::BIT128_BLOCK_SIZE;
        let mut result = vec![];
        for i in 0..block_count {
            let mut block: GAB128 =
                GAB128::clone_from_slice(&payload[i * AESMode::BIT128_BLOCK_SIZE..(i + 1) * AESMode::BIT128_BLOCK_SIZE]);
            cipher.decrypt_block(&mut block);
            if i + 1 == block_count {
                result.extend_from_slice(Pkcs7::unpad(&block).map_err(erx::smp)?);
            } else {
                result.extend_from_slice(&block);
            }
        }

        Ok(result)
    }

    fn cbc_encrypt(&self, key: &[u8], payload: &[u8]) -> Result<Vec<u8>, erx::Erx> {
        // type Aes128CbcEnc = cbc::Encryptor<Aes128>;

        let key = GAB128::from_slice(key);
        let iv = match self {
            AESMode::CBC { iv } => iv.clone(),
            _ => {
                return Err(erx::Erx::new("error call CBC method"));
            },
        };

        let mut buffer = Self::padding_buffer(payload);
        let encoder: cbc::Encryptor<Aes128> = cbc::Encryptor::new(key, iv.as_slice().into());
        let result = encoder.encrypt_padded_mut::<Pkcs7>(buffer.as_mut_slice(), payload.len()).map_err(erx::smp)?.to_vec();

        Ok(result)
    }

    fn cbc_decrypt(&self, key: &[u8], payload: &[u8]) -> Result<Vec<u8>, erx::Erx> {
        let key = GAB128::from_slice(key);
        let iv = match self {
            AESMode::CBC { iv } => iv.clone(),
            _ => {
                return Err(erx::Erx::new("error call CBC method"));
            },
        };

        let decoder: cbc::Decryptor<Aes128> = cbc::Decryptor::new(key, iv.as_slice().into());
        let mut buffer = payload.to_vec();
        let result = decoder.decrypt_padded_mut::<Pkcs7>(buffer.as_mut_slice()).map_err(erx::smp)?.to_vec();

        Ok(result)
    }

    fn cfb_encrypt(&self, key: &[u8], payload: &[u8]) -> Result<Vec<u8>, erx::Erx> {
        let key = GAB128::from_slice(key);
        let iv = match self {
            AESMode::CFB { iv } => iv.clone(),
            _ => {
                return Err(erx::Erx::new("error call CFB method"));
            },
        };

        let mut buffer = Self::padding_buffer(payload);
        let encryptor: cfb_mode::Encryptor<Aes128> = cfb_mode::Encryptor::new(key, iv.as_slice().into());
        let result = encryptor.encrypt_padded_mut::<Pkcs7>(buffer.as_mut_slice(), payload.len()).map_err(erx::smp)?.to_vec();

        Ok(result)
    }
    fn cfb_decrypt(&self, key: &[u8], payload: &[u8]) -> Result<Vec<u8>, erx::Erx> {
        let key = GAB128::from_slice(key);
        let iv = match self {
            AESMode::CFB { iv } => iv.clone(),
            _ => {
                return Err(erx::Erx::new("error call CFB method"));
            },
        };

        let mut buf = payload.to_vec();
        let dec: cfb_mode::Decryptor<Aes128> = cfb_mode::Decryptor::new(key, iv.as_slice().into());
        let result = dec.decrypt_padded_mut::<Pkcs7>(&mut buf).map_err(erx::smp)?.to_vec();

        Ok(result)
    }

    fn ofb_encrypt(&self, key: &[u8], payload: &[u8]) -> Result<Vec<u8>, erx::Erx> {
        let iv = match self {
            AESMode::OFB { iv } => iv.clone(),
            _ => {
                return Err(erx::Erx::new("error call OFB method"));
            },
        };

        type Aes128Ofb = ofb::Ofb<Aes128>;
        let mut buffer = payload.to_vec();
        let mut cipher = Aes128Ofb::new(key.into(), iv.as_slice().into());
        cipher.apply_keystream(&mut buffer.as_mut_slice());

        Ok(buffer)
    }

    fn ofb_decrypt(&self, key: &[u8], payload: &[u8]) -> Result<Vec<u8>, erx::Erx> {
        self.ofb_encrypt(key, payload)
    }

    fn ctr_encrypt(&self, key: &[u8], payload: &[u8]) -> Result<Vec<u8>, erx::Erx> {
        type Aes128Ctr64LE = ctr::Ctr64LE<Aes128>;
        let iv = match self {
            AESMode::CTR { iv } => iv.clone(),
            _ => {
                return Err(erx::Erx::new("error call CTR method"));
            },
        };
        let mut buffer = payload.to_vec();
        let mut cipher = Aes128Ctr64LE::new(key.into(), iv.as_slice().into());
        cipher.apply_keystream(&mut buffer.as_mut_slice());
        Ok(buffer)
    }

    fn ctr_decrypt(&self, key: &[u8], payload: &[u8]) -> Result<Vec<u8>, erx::Erx> {
        self.ctr_encrypt(key, payload)
    }
}

pub enum RSAPadding {
    PKCS1v15,
    // OAEP,
}

impl RSAPadding {
    fn encrypt(&self, public_key: &str, payload: &[u8]) -> Result<Vec<u8>, erx::Erx> {
        match self {
            RSAPadding::PKCS1v15 => {
                let key: RsaPublicKey = DecodeRsaPublicKey::from_pkcs1_pem(public_key).map_err(erx::smp)?;
                let mut rng = CompatRng::<rand::rngs::ThreadRng>::thread_rng();
                let result = key.encrypt(&mut rng, Pkcs1v15Encrypt, payload).map_err(erx::smp)?;
                Ok(result)
            },
        }
    }

    fn decrypt(&self, private_key: &str, payload: &[u8]) -> Result<Vec<u8>, erx::Erx> {
        match self {
            RSAPadding::PKCS1v15 => {
                let key: RsaPrivateKey = DecodeRsaPrivateKey::from_pkcs1_pem(private_key).map_err(erx::smp)?;
                let result = key.decrypt(Pkcs1v15Encrypt, payload).map_err(erx::smp)?;
                Ok(result)
            },
        }
    }
}

pub enum RSABits {
    K1,
    K2,
    K3,
    K4,
}

impl RSABits {
    pub fn bits(&self) -> usize {
        match self {
            RSABits::K1 => 1024,
            RSABits::K2 => 2048,
            RSABits::K3 => 3072,
            RSABits::K4 => 4096,
        }
    }
}

pub struct RSAUtils;

impl RSAUtils {
    pub fn gen_key_pair(bits: RSABits) -> Result<(String, String), erx::Erx> {
        let mut rng = CompatRng::<rand::rngs::ThreadRng>::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, bits.bits()).map_err(erx::smp)?;
        let public_key = RsaPublicKey::from(&private_key);

        let private_key = private_key.to_pkcs1_pem(Default::default()).map_err(erx::smp)?.to_string();
        let public_key = public_key.to_pkcs1_pem(Default::default()).map_err(erx::smp)?.to_string();

        Ok((private_key, public_key))
    }
}

pub enum Encrypt {
    AES { key: String, mode: AESMode },
    RSA { private_key: String, public_key: String, padding: RSAPadding },
}

impl Encrypt {
    pub fn encrypt(&self, input: &[u8]) -> Result<Vec<u8>, erx::Erx> {
        let result = match self {
            Encrypt::AES { key, mode } => mode.encrypt(key.as_bytes(), input)?,
            Encrypt::RSA { private_key: _, public_key, padding } => padding.encrypt(&public_key, input)?,
        };

        Ok(result)
    }

    pub fn decrypt(&self, input: &[u8]) -> Result<Vec<u8>, erx::Erx> {
        let result = match self {
            Encrypt::AES { key, mode } => mode.decrypt(key.as_bytes(), input)?,
            Encrypt::RSA { private_key, public_key: _, padding } => padding.decrypt(&private_key, input)?,
        };

        Ok(result)
    }

    pub fn encrypt_string_base64(&self, input: &str) -> Result<String, erx::Erx> {
        let end = self.encrypt(input.as_bytes())?;
        let hex = base64::prelude::BASE64_STANDARD.encode(&end);
        Ok(hex)
    }

    pub fn decrypt_string_base64(&self, input: &str) -> Result<Vec<u8>, erx::Erx> {
        let decoded = base64::prelude::BASE64_STANDARD.decode(input.as_bytes()).map_err(erx::smp)?;
        self.decrypt(&decoded)
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;

    static PRINT_INFO: bool = false;

    #[test]
    fn test_ecb_encrypt() {
        let key = "1234567890123456";
        let input = "1234567890123456".as_bytes();
        let c = Encrypt::AES { key: key.to_string(), mode: AESMode::ECB };
        let end = c.encrypt(input).unwrap();
        let ded = c.decrypt(&end).unwrap();

        assert_eq!(input, ded);

        if PRINT_INFO {
            println!("{:?}", base64::prelude::BASE64_STANDARD.encode(&end));
            println!("{:?}", String::from_utf8(ded).unwrap());
        }
    }

    #[test]
    fn test_cbc_encrypt() {
        let key = "1234567890123456";
        let input = "1234567890123456".as_bytes();
        let c = Encrypt::AES { key: key.to_string(), mode: AESMode::CBC { iv: key.as_bytes().to_vec() } };
        let end = c.encrypt(input).unwrap();
        let ded = c.decrypt(&end).unwrap();

        assert_eq!(input, ded);

        if PRINT_INFO {
            println!("{:?}", base64::prelude::BASE64_STANDARD.encode(&end));
            println!("{:?}", String::from_utf8(ded).unwrap());
        }
    }

    #[test]
    fn test_cfb_encrypt() {
        let key = "1234567890123456";
        let input = "1234567890123456".as_bytes();
        let c = Encrypt::AES { key: key.to_string(), mode: AESMode::CFB { iv: key.as_bytes().to_vec() } };
        let end = c.encrypt(input).unwrap();
        let ded = c.decrypt(&end).unwrap();

        assert_eq!(input, ded);

        if PRINT_INFO {
            println!("{:?}", base64::prelude::BASE64_STANDARD.encode(&end));
            println!("{:?}", String::from_utf8(ded).unwrap());
        }
    }

    #[test]
    fn test_ofb_encrypt() {
        let key = "1234567890123456";
        let input = "1234567890123456".as_bytes();
        let c = Encrypt::AES { key: key.to_string(), mode: AESMode::OFB { iv: key.as_bytes().to_vec() } };
        let end = c.encrypt(input).unwrap();
        let ded = c.decrypt(&end).unwrap();

        assert_eq!(input, ded);

        if PRINT_INFO {
            println!("{:?}", base64::prelude::BASE64_STANDARD.encode(&end));
            println!("{:?}", String::from_utf8(ded).unwrap());
        }
    }

    #[test]
    fn test_ctr_encrypt() {
        let key = "1234567890123456";
        let input = "1234567890123456".as_bytes();
        let c = Encrypt::AES { key: key.to_string(), mode: AESMode::CTR { iv: key.as_bytes().to_vec() } };
        let end = c.encrypt(input).unwrap();
        let ded = c.decrypt(&end).unwrap();

        assert_eq!(input, ded);

        if PRINT_INFO {
            println!("{:?}", base64::prelude::BASE64_STANDARD.encode(&end));
            println!("{:?}", String::from_utf8(ded).unwrap());
        }
    }

    #[test]
    fn test_ras_pairs() {
        let (pri, pbb) = RSAUtils::gen_key_pair(RSABits::K1).unwrap();

        assert!(pri.len() > 10);
        assert!(pbb.len() > 10);
        assert_ne!(pri, pbb);

        if PRINT_INFO {
            println!("{:?}", pri);
            println!("{:?}", pbb);
        }
    }

    #[test]
    fn test_rsa() {
        let input = "1234567890123456".as_bytes();

        let (pri, pbb) = RSAUtils::gen_key_pair(RSABits::K2).unwrap();

        let c = Encrypt::RSA { private_key: pri, public_key: pbb, padding: RSAPadding::PKCS1v15 };

        let end = c.encrypt(input).unwrap();
        let ded = c.decrypt(&end).unwrap();

        assert_eq!(input, ded);

        if PRINT_INFO {
            println!("{:?}", base64::prelude::BASE64_STANDARD.encode(&end));
            println!("{:?}", String::from_utf8(ded).unwrap());
        }
    }
}
