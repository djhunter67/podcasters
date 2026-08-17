/// Business logic lives here
use actix_web::{HttpResponse, get, http::header::ContentType};
use askama::Template;

#[derive(Template)]
#[template(path = "parts/schedule.part.html")]
struct ScheduleTemplate<'a> {
    content: Vec<&'a str>,
    user: &'a str,
}

#[derive(Template)]
#[template(path = "parts/testimonials.parts.html")]
struct TestimonialTemplate<'a> {
    content: Vec<&'a str>,
    user: &'a str,
}

#[derive(Template)]
#[template(path = "parts/finances.part.html")]
struct FinancesTemplate<'a> {
    content: Vec<&'a str>,
    user: &'a str,
}

#[derive(Template)]
#[template(path = "parts/contact.parts.html")]
struct ContactTemplate<'a> {
    content: Vec<&'a str>,
    user: &'a str,
}

#[get("/schedule")]
pub async fn schedule() -> HttpResponse {
    let open_dates: &str = "All the open and available dates";
    let closed_dates: &str = "These dates have been reserved";
    let canceled: &str = "Cancellations";
    let template = ScheduleTemplate {
        content: [open_dates, closed_dates, canceled].to_vec(),
        user: "logged in user",
    };

    let template = template.render().expect("About page render error");

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(template)
}

#[get("/testimonials")]
pub async fn testimonials() -> HttpResponse {
    let customer_feedback: &str = "The company started is great!";
    let ratings: &str = "Five Stars";
    let dates_of_service: &str = "A DateTime object";
    let template = TestimonialTemplate {
        content: [customer_feedback, ratings, dates_of_service].to_vec(),
        user: "logged in user",
    };

    let template = template.render().expect("About page render error");

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(template)
}

#[get("/finances")]
pub async fn finances() -> HttpResponse {
    let finances_benefit: &str = "Hiring quality employees!";
    let financial_aid: &str = "To be determined";
    let customer_value: &str = "The value provided to a customer from our services";
    let template = FinancesTemplate {
        content: [finances_benefit, financial_aid, customer_value].to_vec(),
        user: "logged in user",
    };

    let template = template.render().expect("About page render error");

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(template)
}

#[get("/contact")]
pub async fn contact() -> HttpResponse {
    let business_contact: &str = "(623) 800-2580";
    let personal_contact: &str = "(623) 555-2560";
    let business_email: &str = "nahan@sundaylifeservices.com";
    let template = ContactTemplate {
        content: [business_contact, personal_contact, business_email].to_vec(),
        user: "logged in user",
    };

    let template = template.render().expect("About page render error");

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(template)
}
