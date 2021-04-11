extern crate winrt_notification;

use winrt_notification::{
    Duration, Toast, ToastAction, ToastActivatedEventArgs, ToastNotification,
};

fn main() {
    let toast = Toast::new(Toast::POWERSHELL_APP_ID);
    let toast_result = toast
        .title("What picture is your favorate?")
        .text1("Your answer is confidential")
        .action(&ToastAction::new().text("The bird").arguments("bird"))
        .action(&ToastAction::new().text("The flower").arguments("flower"))
        .duration(Duration::Short)
        .sound(None)
        .show_with_action(
            |_sender: &ToastNotification, args: &ToastActivatedEventArgs| {
                if args.Arguments()? == "bird" {
                    Toast::new(Toast::POWERSHELL_APP_ID)
                        .text1("Really the bird?")
                        .show()
                        .expect("notification failed");
                }
                Ok(())
            }
        );

    // Actions only work on running applications
    // (The mechanisms windows provides to avoid that are not set up with this crate)
    std::thread::sleep(std::time::Duration::from_millis(10000));
    toast_result
        // silently consume errors
        .expect("notification failed");
}
