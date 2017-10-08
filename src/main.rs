
#[macro_use]
mod objc {
    pub enum Id {}
    pub enum Sel {}

    #[link(name = "objc")]
    extern "C" {
        pub fn objc_getClass(name: *const u8) -> Option<&Id>;
        pub fn objc_msgSend(id: *const Id, op: *const Sel, ...) -> Option<&Id>;
        pub fn sel_registerName(name: *const u8) -> Option<&Sel>;
    }

    macro_rules! objc_class {
        ($n:expr) => {
            unsafe {
                objc::objc_getClass(concat!($n, "\0").as_ptr()).unwrap()
            }
        }
    }

    macro_rules! objc_sel {
        ($n:expr) => {
            unsafe {
                objc::sel_registerName(concat!($n, "\0").as_ptr()).unwrap()
            }
        }
    }

    macro_rules! objc_msg_send {
        ($self:expr, $op:expr) => {
            unsafe { objc::objc_msgSend($self, $op) }
        };
        ($self:expr, $op:expr, $($args:expr),*) => {
            unsafe { objc::objc_msgSend($self, $op, $($args,)*) }
        };
    }
}

mod ns {
    use std::ffi;
    use std::mem;
    use std::string;
    use objc;

    pub type String = objc::Id;

    impl String {
        pub fn new(s: &str) -> &String {
            let cls = objc_class!("NSString");
            let alloc = objc_sel!("alloc");
            let result = objc_msg_send!(cls, alloc).unwrap();
            let op = objc_sel!("initWithBytes:length:encoding:");
            objc_msg_send!(result, op, s.as_ptr(), s.len(), 4).unwrap()
        }
    }

    impl<'a> From<&'a String> for string::String {
        fn from(ns_string: &'a String) -> Self {
            let op = objc_sel!("UTF8String");
            let c_string_bytes = objc_msg_send!(ns_string, op).unwrap();
            unsafe {
                ffi::CString::from_raw(mem::transmute::<&objc::Id, *mut i8>(c_string_bytes))
                    .into_string()
                    .unwrap()
            }
        }
    }

    pub type Application = objc::Id;

    pub enum ApplicationActivationPolicy {
        Regular = 0,
    }

    impl Application {
        pub fn shared<'a>() -> &'a Application {
            let cls = objc_class!("NSApplication");
            let op = objc_sel!("sharedApplication");
            objc_msg_send!(cls, op).unwrap()
        }
        pub fn set_activation_policy(&self, policy: ApplicationActivationPolicy) -> bool {
            let op = objc_sel!("setActivationPolicy:");
            objc_msg_send!(self, op, policy as u32).is_some()
        }
        pub fn finish_launching(&self) {
            let op = objc_sel!("finishLaunching");
            objc_msg_send!(self, op);
        }
    }

    pub type Bundle = objc::Id;

    impl Bundle {
        pub fn main<'a>() -> &'a Bundle {
            let cls = objc_class!("NSBundle");
            let op = objc_sel!("mainBundle");
            objc_msg_send!(cls, op).unwrap()
        }
        pub fn object(&self, key: &str) -> Option<&objc::Id> {
            let op = objc_sel!("objectForInfoDictionaryKey:");
            objc_msg_send!(self, op, String::new(key))
        }
    }

    pub type ProcessInfo = objc::Id;

    impl ProcessInfo {
        pub fn process_info<'a>() -> &'a ProcessInfo {
            let cls = objc_class!("NSProcessInfo");
            let op = objc_sel!("processInfo");
            objc_msg_send!(cls, op).unwrap()
        }
        pub fn process_name(&self) -> &String {
            let op = objc_sel!("processName");
            objc_msg_send!(self, op).unwrap()
        }
    }

    #[link(name = "AppKit", kind = "framework")]
    #[link(name = "Foundation", kind = "framework")]
    extern "C" {
        // static NSApp: *mut Application;
    }
}

fn main() {
    let app = ns::Application::shared();
    app.set_activation_policy(ns::ApplicationActivationPolicy::Regular);

    let main_bundle = ns::Bundle::main();
    let app_name = Option::None
        .or_else(|| main_bundle.object("CFBundleDisplayName"))
        .or_else(|| main_bundle.object("CFBundleName"))
        .unwrap_or_else(|| ns::ProcessInfo::process_info().process_name());

    println!("{:?}", String::from(app_name));

    app.finish_launching();
}
