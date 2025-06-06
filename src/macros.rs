 
#[macro_export]
macro_rules! s {
    ($s:expr) => {
        String::from($s)
    };
}
 
#[macro_export]
macro_rules! ms {
    ($x:ident) => {
        stringify!($x)
    };
}
 
#[macro_export]
macro_rules! mst {
    ($x:ident) => {
        String::from(stringify!($x))
    };
}

// to optional string
#[macro_export]
macro_rules! tos {
    ($e:expr) => {
        Some($e.to_string())
    };
    ($($e:expr),+) => {
        Some(tos_helper!($($e),+))
    };
}

// to string
#[macro_export]
macro_rules! ts {
    ($e:expr) => {
        $e.to_string()
    };
    ($($e:expr),+) => {
        format!("{}", ts_helper!($($e),+))
    };
}

#[macro_export]
#[allow(unused_macros)]
// Helper macro to handle the recursion
macro_rules! ts_helper {
    ($e:expr) => {
        $e.to_string()
    };
    ($e:expr, $($rest:expr),+) => {
        format!("{}{}", $e, ts_helper!($($rest),+))
    };
}

// is all value none
#[macro_export]
macro_rules! all_none {
    ($e:expr) => {
        $e.is_none()
    };
    ($e:expr, $($rest:expr),+) => {
        $e.is_none() || all_none!($($rest),+)
    };
}

// convert Result<T,_> to Result<T,String>
#[macro_export]
macro_rules! result_message {
    ($s:expr) => {
        match $s {
            Ok(r) => Ok(r),
            Err(err) => Err(err.to_string()),
        }
    };
}

#[macro_export]
macro_rules! ternary {
    ($condition:expr, $true_value:expr, $false_value:expr) => {
        if $condition { $true_value } else { $false_value }
    };
}

#[macro_export]
macro_rules! try_or_return {
    ($expr:expr) => {
        match $expr {
            Ok(value) => value,
            Err(e) => {
                error!("{}", e);
                return;
            },
        }
    };
}

///
#[macro_export]
macro_rules! its_service {
    () => {
        pub async fn its_service() {
            ringm::serviced!();
        }
    };
    ($e:expr) => {
        pub async fn its_service() {
            ringm::serviced!(stringify!($e));
        }
    };
}

#[macro_export]
macro_rules! hey_service {
    ($e:ident) => {{ $e::its_service().await }};
}


#[macro_export]
macro_rules! use_seaorm_min {
    () => {
        #[allow(unused_imports)]
        use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, Condition, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder};
    };
}

