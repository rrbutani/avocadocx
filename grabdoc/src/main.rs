use google_drive::Client;

fn main() {
    let drive = Client::new(
        String::from("client-id"),
        String::from("client-secret"),
        String::from("redirect-uri"),
        String::from("token"),
        String::from("refresh-token"),
    );
    drive.files().export("", "application/vnd.openxmlformats-officedocument.wordprocessingml.document");
}
