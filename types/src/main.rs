enum CanNeverExist {}
fn never_returns() -> CanNeverExist {
    CanNeverExist {}
}

fn main() {
    let value = never_returns();
}