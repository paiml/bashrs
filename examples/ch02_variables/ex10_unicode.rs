// Chapter 2, Example 10: Unicode in Variable Values
// Rash supports Unicode in string values

fn main() {
    let greeting_ja = "こんにちは";
    let greeting_ru = "Привет";
    let greeting_ar = "مرحبا";
    let emoji = "🚀 Deploying...";

    echo("Unicode support:");
    echo(greeting_ja);
    echo(greeting_ru);
    echo(greeting_ar);
    echo(emoji);
}

fn echo(msg: &str) {}
