use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use std::env;

pub fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

pub fn get_invoices() -> Result<Vec<::models::invoice::DbInvoice>, String> {
    let connection = establish_connection();
    let results = ::schema::invoices::table
        .load::<::models::invoice::DbInvoice>(&connection)
        .expect("Error loading invoices");

    info!("Displaying {} invoices: \n {:?}", results.len(), results);

    Ok(results)
}

pub fn insert_invoice(
    inv: &::models::invoice::Invoice,
) -> Result<::models::invoice::DbInvoice, ::diesel::result::Error> {
    info!("Inserting {:?}", inv);
    let connection = establish_connection();
    diesel::insert(inv)
        .into(::schema::invoices::table)
        .get_result(&connection)
}

pub fn get_unpaid_invoices() -> Result<Vec<::models::invoice::DbInvoice>, String> {
    let invoices: Vec<::models::invoice::DbInvoice> = get_invoices().unwrap();
    let payments: Vec<::models::payment::DbPayment> = get_payments().unwrap();

    // TODO: find exact matches
    // * where invoices.number is a substring of payments.invoice_number
    // * and invoices.sirname is a substring of payments.subject
    // * and invoices.name is a substring of payments.subject
    let exact_matches = {
        let paid_invoices: Vec<::models::invoice::DbInvoice> = Vec::new();

        // for invoice in invoices.iter() {
        //     for payment in payments.iter() {
        //         if payment.subject.contains(invoice.sirname) &&
        //             payment.subject.contains(invoice.name) {

        //             }
        //     }
        // }

        paid_invoices
    };

    let unpaid_invoices = invoices
        .iter()
        .filter(|&x| exact_matches.iter().position(|y| x == y) == None)
        .cloned()
        .collect();

    // TODO: add all unpaid invoices
    Ok(unpaid_invoices)
}

pub fn get_payments() -> Result<Vec<::models::payment::DbPayment>, String> {
    let connection = establish_connection();
    let results = ::schema::payments::table
        .load::<::models::payment::DbPayment>(&connection)
        .expect("Error loading payments");

    info!("Displaying {} payments: \n {:?}", results.len(), results);

    Ok(results)
}

pub fn insert_payments(
    payments: Vec<::models::payment::Payment>,
) -> Result<Vec<::models::payment::DbPayment>, ::diesel::result::Error> {
    info!("Inserting {:?}", payments);
    let connection = establish_connection();
    let mut db_payments: Vec<::models::payment::DbPayment> = Vec::new();

    for payment in payments.iter() {
        match {
            diesel::insert(payment)
                .into(::schema::payments::table)
                .get_result(&connection)
        } {
            Ok(db_payment) => {
                db_payments.push(db_payment);
            }
            Err(e) => {
                return Err(e);
            }
        };
    }

    Ok(db_payments)
}
