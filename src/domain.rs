use unicode_segmentation::UnicodeSegmentation;
use validator::ValidateEmail;
pub struct SubscriberName(String);

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl SubscriberName {
    pub fn parse(s: String) -> Result<SubscriberName, Box<dyn std::error::Error>> {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 256;
        let forbidden_characters = [
            '/', '(', ')', '<', '>', ']', '[', '{', '}', ':', '"', '\\', '{', '}',
            ];
            let contains_forbidden_characters = s.chars().any(|c| forbidden_characters.contains(&c));
            if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
                Err(format!("{} is not a valid subscriber name", s).into())
            } else {
                Ok(Self(s))
        }
    }
}
pub struct SubscriberEmail(String);

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl SubscriberEmail {
    pub fn parse(s: String) -> Result<SubscriberEmail, Box<dyn std::error::Error>> {
        match s.validate_email() {
            true => Ok(Self(s)),
            false => Err(format!("{} is not a valid subscriber email", s).into()),
        }
    }
}

pub struct NewSubscriber{
    pub name:SubscriberName,
    pub email:SubscriberEmail
}