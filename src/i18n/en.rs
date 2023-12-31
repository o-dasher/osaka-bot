use rusty18n::{r, I18NFallback};

use super::osaka_i_18_n::{
    self,
    errors::Errors,
    fun::{coinflip::Coinflip, Fun},
    user::avatar::{footer::Footer, Avatar},
    OsakaI18N,
};

impl I18NFallback for OsakaI18N {
    fn fallback() -> Self {
        Self {
            errors: Errors {
                unexpected: r!("Heh? Something unexpected happened with my brain."),
            },
            user: osaka_i_18_n::user::User {
                avatar: Avatar {
                    footer: Footer {
                        eq: r!("Nice, yourself!"),
                        other: r!("They are the..."),
                    },
                },
            },
            fun: Fun {
                coinflip: Coinflip {
                    show: r!("I flip a coin and it lands on..."),
                    heads: r!("Heads"),
                    tails: r!("Tails"),
                },
            },
        }
    }
}
