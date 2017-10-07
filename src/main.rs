
mod objc {
    pub enum Id {}
    pub enum Sel {}

    #[link(name = "objc")]
    extern "C" {
        fn objc_getClass(name: *const u8) -> Option<&Id>;
        pub fn objc_msgSend(id: *const Id, op: *const Sel, ...) -> Option<&Id>;
        fn sel_registerName(name: *const u8) -> *const Sel;
    }

    pub fn class(name: &str) -> &Id {
        unsafe { objc_getClass(name.as_ptr()).unwrap() }
    }

    pub fn selector(name: &str) -> &Sel {
        unsafe { &*sel_registerName(name.as_ptr()) }
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
            let cls = objc::class("NSString\0");
            let alloc = objc::selector("alloc\0");
            unsafe {
                let string = objc::objc_msgSend(cls, alloc).unwrap();
                let op = objc::selector("initWithBytes:length:encoding:\0");
                objc::objc_msgSend(string, op, s.as_ptr(), s.len(), 4).unwrap()
            }
        }
    }

    impl<'a> From<&'a String> for string::String {
        fn from(ns_string: &'a String) -> Self {
            let op = objc::selector("UTF8String\0");
            unsafe {
                let c_string_bytes = objc::objc_msgSend(ns_string, op).unwrap();
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
            let cls = objc::class("NSApplication\0");
            let op = objc::selector("sharedApplication\0");
            unsafe { objc::objc_msgSend(cls, op).unwrap() }
        }
        pub fn set_activation_policy(&self, policy: ApplicationActivationPolicy) -> bool {
            let op = objc::selector("setActivationPolicy:\0");
            unsafe { objc::objc_msgSend(self, op, policy as u32).is_some() }
        }
        pub fn finish_launching(&self) {
            let op = objc::selector("finishLaunching\0");
            unsafe {
                objc::objc_msgSend(self, op);
            }
        }
    }

    pub type Bundle = objc::Id;

    impl Bundle {
        pub fn main<'a>() -> &'a Bundle {
            let cls = objc::class("NSBundle\0");
            let op = objc::selector("mainBundle\0");
            unsafe { objc::objc_msgSend(cls, op).unwrap() }
        }
        pub fn object(&self, key: &str) -> Option<&objc::Id> {
            let op = objc::selector("objectForInfoDictionaryKey:\0");
            unsafe { objc::objc_msgSend(self, op, String::new(key)) }
        }
    }

    pub type ProcessInfo = objc::Id;

    impl ProcessInfo {
        pub fn process_info<'a>() -> &'a ProcessInfo {
            let cls = objc::class("NSProcessInfo\0");
            let op = objc::selector("processInfo\0");
            unsafe { objc::objc_msgSend(cls, op).unwrap() }
        }
        pub fn process_name(&self) -> &String {
            let op = objc::selector("processName\0");
            unsafe { objc::objc_msgSend(self, op).unwrap() }
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
