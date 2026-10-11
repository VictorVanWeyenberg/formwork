use std::cmp::{Ordering, PartialOrd};
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::{AddAssign, SubAssign};
use std::str::FromStr;

use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug)]
pub struct Wallet {
    copper: Cost,
    silver: Cost,
    electrum: Cost,
    gold: Cost,
    platinum: Cost,
}

impl Wallet {
    pub fn empty() -> Wallet {
        Wallet {
            copper: Cost::new(0, Coin::Copper),
            silver: Cost::new(0, Coin::Silver),
            electrum: Cost::new(0, Coin::Electrum),
            gold: Cost::new(0, Coin::Gold),
            platinum: Cost::new(0, Coin::Platinum),
        }
    }

    fn from_coppers(mut copper: usize) -> Wallet {
        let platinum = copper / 1000;
        copper -= platinum * 1000;
        let gold = copper / 100;
        copper -= gold * 100;
        let electrum = copper / 50;
        copper -= electrum * 50;
        let silver = copper / 10;
        copper -= silver * 10;
        Wallet::new(copper, silver, electrum, gold, platinum)
    }

    pub fn new(
        copper: usize,
        silver: usize,
        electrum: usize,
        gold: usize,
        platinum: usize,
    ) -> Wallet {
        Wallet {
            copper: Cost::new(copper, Coin::Copper),
            silver: Cost::new(silver, Coin::Silver),
            electrum: Cost::new(electrum, Coin::Electrum),
            gold: Cost::new(gold, Coin::Gold),
            platinum: Cost::new(platinum, Coin::Platinum),
        }
    }

    pub fn add(&mut self, cost: Cost) {
        match &cost.coin {
            Coin::Copper => self.copper += cost.amount,
            Coin::Silver => self.silver += cost.amount,
            Coin::Electrum => self.electrum += cost.amount,
            Coin::Gold => self.gold += cost.amount,
            Coin::Platinum => self.platinum += cost.amount,
        }
    }

    pub fn subtract(&mut self, cost: Cost) -> Result<(), Wallet> {
        let wallet_coppers = self.to_coppers();
        let cost_coppers = cost.to_coppers();
        if cost_coppers > wallet_coppers {
            Err(Wallet::from_coppers(cost_coppers - wallet_coppers))
        } else {
            let wallet = Wallet::from_coppers(wallet_coppers - cost_coppers);
            self.copper = wallet.copper;
            self.silver = wallet.silver;
            self.electrum = wallet.electrum;
            self.gold = wallet.gold;
            self.platinum = wallet.platinum;
            Ok(())
        }
    }

    fn to_coppers(&self) -> usize {
        self.copper.to_coppers()
            + self.silver.to_coppers()
            + self.gold.to_coppers()
            + self.electrum.to_coppers()
            + self.platinum.to_coppers()
    }
}

#[derive(Debug, Clone)]
pub struct Cost {
    amount: usize,
    coin: Coin,
}

impl Serialize for Cost {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{} {}", self.amount, &self.coin))
    }
}

impl<'de> Deserialize<'de> for Cost {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let cost_regex = Regex::new("^(?<amount>\\d+)\\s(?<coin>cp|sp|ep|gp|pp)$").expect("");
        if let Some(captures) = cost_regex.captures(&String::deserialize(deserializer)?) {
            let amount = captures["amount"].parse::<usize>().unwrap();
            let coin = Coin::from_str(&captures["coin"]).unwrap();
            Ok(Cost { amount, coin })
        } else {
            Err(serde::de::Error::custom("Unable to deserialize cost."))
        }
    }
}

impl AddAssign<usize> for Cost {
    fn add_assign(&mut self, rhs: usize) {
        self.amount += rhs
    }
}

impl SubAssign<usize> for Cost {
    fn sub_assign(&mut self, rhs: usize) {
        self.amount -= rhs
    }
}

impl PartialEq<Self> for Cost {
    fn eq(&self, other: &Self) -> bool {
        self.to_coppers() == other.to_coppers()
    }
}

impl Eq for Cost {}

impl PartialOrd<Self> for Cost {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Cost {
    fn cmp(&self, other: &Cost) -> Ordering {
        self.to_coppers().cmp(&other.to_coppers())
    }
}

impl Hash for Cost {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.amount.hash(state);
        self.coin.hash(state);
    }
}

impl Cost {
    pub fn new(amount: usize, coin: Coin) -> Cost {
        Cost { amount, coin }
    }

    fn to_coppers(&self) -> usize {
        match self.coin {
            Coin::Copper => self.amount,
            Coin::Silver => self.amount * 10,
            Coin::Electrum => self.amount * 50,
            Coin::Gold => self.amount * 100,
            Coin::Platinum => self.amount * 1000,
        }
    }

    pub fn amount(&self) -> &usize {
        &self.amount
    }

    pub fn coin(&self) -> &Coin {
        &self.coin
    }
}

#[derive(Debug, Clone, Hash)]
pub enum Coin {
    Copper,
    Silver,
    Electrum,
    Gold,
    Platinum,
}

impl Display for Coin {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Coin::Copper => "cp",
                Coin::Silver => "sp",
                Coin::Electrum => "ep",
                Coin::Gold => "gp",
                Coin::Platinum => "pp",
            }
        )
    }
}

impl FromStr for Coin {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "cp" {
            Ok(Coin::Copper)
        } else if s == "sp" {
            Ok(Coin::Silver)
        } else if s == "ep" {
            Ok(Coin::Electrum)
        } else if s == "gp" {
            Ok(Coin::Gold)
        } else if s == "pp" {
            Ok(Coin::Platinum)
        } else {
            Err(format!("Unrecognized coin type: {}.", s))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::inventory::wallet::{Coin, Cost, Wallet};

    #[test]
    fn one_copper_from_one_platinum() {
        let mut wallet = Wallet::new(0, 0, 0, 0, 1);
        wallet
            .subtract(Cost::new(1, Coin::Copper))
            .expect("Unexpected error");
        assert_eq!(wallet.copper.amount, 9);
        assert_eq!(wallet.silver.amount, 4);
        assert_eq!(wallet.electrum.amount, 1);
        assert_eq!(wallet.gold.amount, 9);
        assert_eq!(wallet.platinum.amount, 0);
    }

    #[test]
    fn hundred_copper_from_random_wallet() {
        let mut wallet = Wallet::new(23, 16, 2, 5, 1);
        let before_coppers = wallet.to_coppers();
        wallet
            .subtract(Cost::new(100, Coin::Copper))
            .expect("Unexpected error");
        let after_coppers = wallet.to_coppers();
        assert_eq!(before_coppers - after_coppers, 100);
    }

    #[test]
    fn thousand_copper_from_random_wallet() {
        let mut wallet = Wallet::new(23, 16, 2, 5, 1);
        let before_coppers = wallet.to_coppers();
        wallet
            .subtract(Cost::new(1000, Coin::Copper))
            .expect("Unexpected error");
        let after_coppers = wallet.to_coppers();
        assert_eq!(before_coppers - after_coppers, 1000);
    }

    #[test]
    fn ten_thousand_copper_from_one_platinum_wallet() {
        let mut wallet = Wallet::new(0, 0, 0, 0, 1);
        match wallet.subtract(Cost::new(10000, Coin::Copper)) {
            Ok(_) => panic!("Unexpected Ok"),
            Err(wallet) => {
                assert_eq!(wallet.platinum.amount, 9);
                assert_eq!(wallet.gold.amount, 0);
                assert_eq!(wallet.electrum.amount, 0);
                assert_eq!(wallet.silver.amount, 0);
                assert_eq!(wallet.copper.amount, 0);
            }
        }
    }

    #[test]
    fn ten_thousand_copper_from_one_platinum_and_one_electrum_wallet() {
        let mut wallet = Wallet::new(0, 0, 0, 0, 1);
        match wallet.subtract(Cost::new(10050, Coin::Copper)) {
            Ok(_) => panic!("Unexpected Ok"),
            Err(wallet) => {
                assert_eq!(wallet.platinum.amount, 9);
                assert_eq!(wallet.gold.amount, 0);
                assert_eq!(wallet.electrum.amount, 1);
                assert_eq!(wallet.silver.amount, 0);
                assert_eq!(wallet.copper.amount, 0);
            }
        }
    }
}
