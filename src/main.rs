use std::{time::Duration, thread::sleep};

use dacal:: Dacal;

fn main() {
    for d in dacal::devices() {
        println!("{}", d.id);
        d.access_slot(25).expect("Slot to be accesible");
    }

    sleep(Duration::from_secs(5));

    let d = Dacal::from_id(45540).unwrap();
    d.retract_arm().expect("Arm to be retractable");
}