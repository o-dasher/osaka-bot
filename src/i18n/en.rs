use rusty18n::{r, I18NDynamicResource, I18NFallback};

use super::osaka_i_18_n::{
    self,
    booru::{
        blacklist::{remove::Remove, Blacklist},
        Booru,
    },
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
                user_missing_permissions: r!("You don't have the required permissions to execute this command at this level of privilege.")
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
            booru: Booru {
                blacklist: Blacklist {
                    blacklisted: r!(|tag| "Sure mistah, blacklisting {tag}!"),
                    everything_blacklisted_already: r!(|tag| "Hey, listen! {tag} is already on the blacklist..."),
                    partial_blacklist: r!(|(was, got)| "Since {was} already was blacklisted, I only blacklisted {got}!"),
                    remove: Remove {
                        removed: r!(|tag| "Ok, then! {tag} is not blacklisted anymore."),
                        failed: r!(|tag| "Hey, it seems that {tag} is not being blacklisted here!")
                    }
                },
            },
        }
    }
}
