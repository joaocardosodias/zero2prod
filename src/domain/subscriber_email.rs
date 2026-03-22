use validator::ValidateEmail;
#[derive(Debug, Clone)]
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

#[cfg(test)]
mod tests {
    use super::SubscriberEmail;
    use claims::assert_err;
    use fake::Fake;
    use fake::faker::internet::en::SafeEmail;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[derive(Debug, Clone)]
    struct ValidateEmailFixture(pub String);
    impl quickcheck::Arbitrary for ValidateEmailFixture {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut rng = StdRng::seed_from_u64(u64::arbitrary(g));
            let email = SafeEmail().fake_with_rng(&mut rng);
            Self(email)
        }
    }
    #[quickcheck_macros::quickcheck]
    fn valid_emails_area_parsed_successfully(email: ValidateEmailFixture) -> bool {
        SubscriberEmail::parse(email.0).is_ok()
    }
}
