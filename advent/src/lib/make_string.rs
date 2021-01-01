pub trait MakeString {
    fn mk_string(&mut self, sep: &str) -> String;
}

impl<I, T: ToString> MakeString for I
where I: Iterator<Item=T> {
    fn mk_string(&mut self, sep: &str) -> String {
        let mut r = String::new();

        if let Some(t) = self.next() {
            r.push_str(&t.to_string());
        }

        while let Some(t) = self.next() {
            r.push_str(sep);
            r.push_str(&t.to_string());
        }

        r
    }
}