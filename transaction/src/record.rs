pub struct Record {
    content: String
}

impl Into<String> for Record {
    fn into(self) -> String {
        self.content
    }
}

