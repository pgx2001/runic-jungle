use candid::Principal;

pub fn trap_if_anonymous() -> Result<(), String> {
    let caller = ic_cdk::caller();
    if caller == Principal::anonymous() {
        Err(String::from("Anonymous Identity"))
    } else {
        Ok(())
    }
}
