

// Text-to-Speech
pub struct TTS { 
    texts: Vec<String>,
}


// Speech-to-Text (STT) 
// ASR（Automatic Speech Recognition）
pub struct STT {

}

impl TTS {
    pub fn new(texts:Vec<String>) -> Self {
        Self {
            texts,
        }
    }

    pub fn texts(&self) -> &Vec<String> {
        &self.texts
    }

    pub fn set_texts(&mut self, texts: Vec<String>) {
        self.texts = texts;
    }

    // pub async speech(&self) -> Result<Vec<u8>, erx::Erx> {
        
    // }
}