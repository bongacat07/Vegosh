mod server;
mod protocol;

fn main() {
    server::initialise_server().unwrap();
}
