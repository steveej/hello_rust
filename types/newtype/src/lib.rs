struct Miles(pub f64);
struct Kilometers(pub f64);

impl Miles {
    fn as_kilometers(&self) -> Kilometers {
        Kilometers { 0: self.0 * 1.6 }
    }
}
impl Kilometers {
    fn as_miles(&self) -> Miles {
        Miles { 0: self.0 / 1.6 }
    }
}

struct Route {
    distance: Miles,
}

impl Route {
    fn new(mi: Miles) -> Route {
        Route { distance: mi }
    }

    fn are_we_there_yet(&self, distance_travelled: Miles) -> bool {
        self.distance.0 <= distance_travelled.0
    }
}

pub fn main() -> ! {
    let distance = Miles { 0: 100.0 };
    let route_miles = Route { distance };
    let travelled = Kilometers { 0: 100.0 };
    let arrived = route_miles.are_we_there_yet( travelled.as_miles() );
    println!("Are we there yet? {}", arrived);
}