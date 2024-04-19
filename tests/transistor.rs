#[test]
fn print_display_list() {
    transistor::print_displays();
}

#[test]
fn create_server() {
    let server = transistor::Server::new(2426);
}

#[test]
fn create_client() {
    let client = transistor::Client::new();
}
