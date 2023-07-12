use std::fmt::Debug;
pub fn check_error_music<RT, ET: Debug>(result: Result<RT, ET>) {
    match result {
        Err(e) => {
            println!("Have some error on music command {:#?}", e);
        }
        _ => {}
    }
}
