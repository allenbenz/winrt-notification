extern crate winrt_notification;

use winrt_notification::{
    InputType,
    Toast,
    ToastWithHandlers,
};

fn main() {
    let toast = Toast::new(Toast::POWERSHELL_APP_ID)
        .title("toast with input")
        .input(InputType::text_with_placeholder("input some text"), "first input")
        .input(InputType::text_with_placeholder("and some more"), "second input");
    ToastWithHandlers::new(toast)
        .on_activate(|args| {
            let input = args.get_user_input().unwrap()?;
            println!("first input is {}", input["first input"]);
            println!("second input is {}", input["second input"]);
            // exit the waiting loop
            std::process::exit(0);
        })
        .show()
        .expect("unable to send notification");
    // wait for activation
    loop {}
}
