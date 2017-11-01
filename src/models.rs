pub mod invoice {
    use schema::invoices;

    #[derive(Queryable, Debug, Clone, PartialEq)]
    pub struct DbInvoice {
        id: i32,
        pub sirname: String,
        pub name: String,
        pub number: String,
        pub total: String,
        pub date: String,
    }

    #[derive(Insertable, Debug, PartialEq)]
    #[table_name = "invoices"]
    pub struct Invoice {
        pub sirname: String,
        pub name: String,
        pub number: String,
        pub total: String,
        pub date: String,
    }

    impl Invoice {
        pub fn new(sirname: &str, name: &str, date: &str, number: &str, total: &str) -> Invoice {
            self::Invoice {
                sirname: sirname.trim().to_string(),
                name: name.trim().to_string(),
                date: date.trim().to_string(),
                number: number.trim().to_string(),
                total: total.trim().to_string().replace("'", ""),
            }
        }
    }
}

pub mod payment {
    use schema::payments;

    #[derive(Queryable, Debug)]
    pub struct DbPayment {
        id: i32,
        date: String,
        number: String,
        unused: String,
        subject: String,
        invoice_number: String,
        amount: String,
    }

    #[derive(Insertable, Debug, PartialEq)]
    #[table_name = "payments"]
    pub struct Payment {
        date: String,
        number: String,
        unused: String,
        subject: String,
        invoice_number: String,
        amount: String,
    }

    impl Payment {
        pub fn new(
            date: &str,
            number: &str,
            unused: &str,
            subject: &str,
            invoice_number: &str,
            amount: &str,
        ) -> Payment {
            self::Payment {
                date: date.to_string(),
                number: number.to_string(),
                unused: unused.to_string(),
                subject: subject.to_string(),
                invoice_number: invoice_number.to_string(),
                amount: amount.to_string(),
            }
        }
    }
}