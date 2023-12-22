#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

mod Target;
use Target::*;

mod Log;

fn main() {
    let Target = Target_type::Get_current();

    Print_line!("Start Xila build");

    Print_line!("Target architecture: {:?}", Target);

    if Target.Get_operating_system() == Operating_system_type::ESP_IDF {
        embuild::espidf::sysenv::output();
    }
}
