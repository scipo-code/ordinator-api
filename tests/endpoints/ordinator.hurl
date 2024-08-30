POST http://127.0.0.1:8080/dotenv::var("ORDINATOR_MAIN_ENDPOINT)")
{
    "message_type":"Tactical",
    "Status":"General"
}

HTTP *
[Asserts]
status >= 200
status <  300
