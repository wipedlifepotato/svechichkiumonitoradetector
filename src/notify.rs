trait Notifier {
    fn send(&self, msg: &str);
}
struct DesktopNotifier;
impl Notifier for DesktopNotifier {
    fn send(&self, msg: &str) {
        todo!("not implemented");
    }
}

//end notifer
