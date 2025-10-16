use hash_with::HashWith;
use std::hash::{Hash, Hasher};

#[cfg(test)]
mod tests {
    use std::hash::DefaultHasher;

    use super::*;

    pub fn hash_f64_bits<H: Hasher>(val: &f64, state: &mut H) {
        val.to_bits().hash(state)
    }

    #[derive(Default, HashWith)]
    struct Foo {
        #[hash_with = "hash_f64_bits"]
        a: f64,
        b: u64,
        #[hash_with({
            let v = self.c.to_bits();
            v
        })]
        c: f64,
        #[hash_with(self.d)]
        d: u64,
    }

    impl Foo {
        pub fn to_hash(&self) -> u64 {
            let mut hasher = DefaultHasher::new();
            self.hash(&mut hasher);
            return hasher.finish();
        }
    }

    #[test]
    /// Test to ensure that the NameValue implementation hashes the value.
    /// The NameValue implementation looks like this: `#[hash_with = "foo"]`
    fn checking_function_hash_with() {
        // Initialization
        let foo_1 = Foo {
            a: 1.0,
            ..Default::default()
        };
        let foo_2 = Foo::default();
        // Compares hash with non-set value
        assert_ne!(foo_1.to_hash(), foo_2.to_hash());
    }

    #[test]
    /// Test to ensure arbitrary value works.
    /// This checks the `#[hash_with( ... )]` notation.
    fn checking_inline_hash_with() {
        // Initialization
        let foo_1 = Foo {
            // Here we check the float implementation
            c: 3.14159,
            ..Default::default()
        };
        let foo_2 = Foo {
            // Here we check the integer simple implementation
            d: 25,
            ..Default::default()
        };
        let foo_3 = Foo::default();
        assert_ne!(foo_1.to_hash(), foo_3.to_hash());
        assert_ne!(foo_2.to_hash(), foo_3.to_hash());
    }

}
