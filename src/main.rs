use otterner::container::Container;

fn main() {
    let mut container = Container::new(
        4096,
        100 * 1024 * 1024,
        20,
        "junk/ubuntu-fs".into(),
        "/bin/bash".to_owned(),
    );

    container.container_creator()
}
