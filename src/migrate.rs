#[allow(unused_imports)]
use tracing::error;

#[macro_export]
macro_rules! migrate_create_tables {
    ( $manager:expr, $($x:ident),+ ) => {
        {
            #[allow(unused_imports)]
            use tracing::{error};

            $(
                let result = $manager.create_table($x::define()).await;
                if result.is_err() {
                    if let Err(e) = &result {
                        error!("{:#?}", e);
                    }
                    return result;
                }
            )*
            Ok(())
        }
    };
}

#[macro_export]
macro_rules! migrate_drop_tables {
    ( $manager:expr, $($x:ident),+ ) => {
        {
            #[allow(unused_imports)]
            use log::{error};

            $(
                let result = $manager.drop_table(Table::drop().table($x::Table).to_owned()).await;
                if result.is_err() {
                    if let Err(e) = &result {
                        error!("{:#?}", e);
                    }

                    return result;
                }
            )*
            Ok(())
        }
    };
}
