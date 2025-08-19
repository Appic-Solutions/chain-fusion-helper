use base64::{engine::general_purpose, Engine as _};
use candid::Principal;
use ic_http_types::{HttpResponse, HttpResponseBuilder};
use serde_bytes::ByteBuf;

use crate::{
    numeric::Erc20TokenAmount,
    state::types::{IcpToken, IcpTokenType},
};

// Helper function to create a mock IcpToken
fn create_mock_token(logo: &str) -> IcpToken {
    IcpToken {
        ledger_id: Principal::anonymous(),
        name: "Test Token".to_string(),
        decimals: 8,
        symbol: "TST".to_string(),
        usd_price: "1.00".to_string(),
        logo: logo.to_string(),
        fee: Erc20TokenAmount::from(10000_u128),
        token_type: IcpTokenType::ICRC1,
        rank: Some(1),
        listed_on_appic_dex: None,
    }
}

// The code snippet to test
fn process_logo_response(token: IcpToken) -> HttpResponse {
    let parts: Vec<&str> = token.logo.splitn(2, ';').collect();
    let content_type = if parts.len() > 0 && parts[0].starts_with("data:") {
        parts[0].strip_prefix("data:").unwrap_or("image/png")
    } else {
        "image/png" // Fallback
    };
    let base64_start = token.logo.find("base64,").map(|pos| pos + 7).unwrap_or(0);
    let base64_data = &token.logo[base64_start..];

    match general_purpose::STANDARD.decode(base64_data) {
        Ok(decoded) => HttpResponseBuilder::ok()
            .header("Content-Type", content_type)
            .body(decoded)
            .build(),
        Err(e) => HttpResponseBuilder::server_error(format!("Base64 decode error: {}", e))
            .body(format!("Base64 decode error: {}", e).as_bytes().to_vec())
            .build(),
    }
}

#[test]
fn test_valid_png_data_url() {
    // Sample PNG data URL (small 1x1 pixel PNG)
    let png_data_url = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAACklEQVR4nGMAAQAABQABDQottAAAAABJRU5ErkJggg==";
    let token = create_mock_token(png_data_url);

    let response = process_logo_response(token);

    // Expected decoded PNG binary
    let expected_body = general_purpose::STANDARD
            .decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAACklEQVR4nGMAAQAABQABDQottAAAAABJRU5ErkJggg==")
            .unwrap();

    assert_eq!(response.status_code, 200);
    assert_eq!(
        response.headers,
        vec![("Content-Type".to_string(), "image/png".to_string())]
    );
    assert_eq!(response.body, ByteBuf::from(expected_body));
}

#[test]
fn test_valid_svg_data_url() {
    // Provided SVG data URL
    let svg_data_url="data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTQ2IiBoZWlnaHQ9IjE0NiIgdmlld0JveD0iMCAwIDE0NiAxNDYiIGZpbGw9Im5vbmUiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyI+CjxyZWN0IHdpZHRoPSIxNDYiIGhlaWdodD0iMTQ2IiByeD0iNzMiIGZpbGw9IiMzQjAwQjkiLz4KPHBhdGggZmlsbC1ydWxlPSJldmVub2RkIiBjbGlwLXJ1bGU9ImV2ZW5vZGQiIGQ9Ik0xNi4zODM3IDc3LjIwNTJDMTguNDM0IDEwNS4yMDYgNDAuNzk0IDEyNy41NjYgNjguNzk0OSAxMjkuNjE2VjEzNS45NEMzNy4zMDg3IDEzMy44NjcgMTIuMTMzIDEwOC42OTEgMTAuMDYwNSA3Ny4yMDUySDE2LjM4MzdaIiBmaWxsPSJ1cmwoI3BhaW50MF9saW5lYXJfMTEwXzYwNCkiLz4KPHBhdGggZmlsbC1ydWxlPSJldmVub2RkIiBjbGlwLXJ1bGU9ImV2ZW5vZGQiIGQ9Ik02OC43NjQ2IDE2LjM1MzRDNDAuNzYzOCAxOC40MDM2IDE4LjQwMzcgNDAuNzYzNyAxNi4zNTM1IDY4Ljc2NDZMMTAuMDMwMyA2OC43NjQ2QzEyLjEwMjcgMzcuMjc4NCAzNy4yNzg1IDEyLjEwMjYgNjguNzY0NiAxMC4wMzAyTDY4Ljc2NDYgMTYuMzUzNFoiIGZpbGw9IiMyOUFCRTIiLz4KPHBhdGggZmlsbC1ydWxlPSJldmVub2RkIiBjbGlwLXJ1bGU9ImV2ZW5vZGQiIGQ9Ik0xMjkuNjE2IDY4LjczNDNDMTI3LjU2NiA0MC43MzM0IDEwNS4yMDYgMTguMzczMyA3Ny4yMDUxIDE2LjMyMzFMNzcuMjA1MSA5Ljk5OTk4QzEwOC42OTEgMTIuMDcyNCAxMzMuODY3IDM3LjI0ODEgMTM1LjkzOSA2OC43MzQzTDEyOS42MTYgNjguNzM0M1oiIGZpbGw9InVybCgjcGFpbnQxX2xpbmVhcl8xMTBfNjA0KSIvPgo8cGF0aCBmaWxsLXJ1bGU9ImV2ZW5vZGQiIGNsaXAtcnVsZT0iZXZlbm9kZCIgZD0iTTc3LjIzNTQgMTI5LjU4NkMxMDUuMjM2IDEyNy41MzYgMTI3LjU5NiAxMDUuMTc2IDEyOS42NDcgNzcuMTc0OUwxMzUuOTcgNzcuMTc0OUMxMzMuODk3IDEwOC42NjEgMTA4LjcyMiAxMzMuODM3IDc3LjIzNTQgMTM1LjkwOUw3Ny4yMzU0IDEyOS41ODZaIiBmaWxsPSIjMjlBQkUyIi8+CjxwYXRoIGQ9Ik04OS4yMjUzIDgyLjMzOTdDODkuMjI1MyA3My43Mzc1IDg0LjA2MjggNzAuNzg3NSA3My43Mzc4IDY5LjU1NDRDNjYuMzYyOCA2OC41NjkxIDY0Ljg4NzggNjYuNjA0NCA2NC44ODc4IDYzLjE2NDdDNjQuODg3OCA1OS43MjUgNjcuMzQ4MSA1Ny41MTI1IDcyLjI2MjggNTcuNTEyNUM3Ni42ODc4IDU3LjUxMjUgNzkuMTQ4MSA1OC45ODc1IDgwLjM3NTMgNjIuNjc1QzgwLjYyMzEgNjMuNDEyNSA4MS4zNjA2IDYzLjkwMjIgODIuMDk4MSA2My45MDIySDg2LjAzMzRDODcuMDE4NyA2My45MDIyIDg3Ljc1NjIgNjMuMTY0NyA4Ny43NTYyIDYyLjE3OTRWNjEuOTMxNkM4Ni43NzA5IDU2LjUyMTMgODIuMzQ1OSA1Mi4zNDQxIDc2LjY5MzcgNTEuODU0NFY0NS45NTQ0Qzc2LjY5MzcgNDQuOTY5MSA3NS45NTYyIDQ0LjIzMTYgNzQuNzI5IDQzLjk4OTdINzEuMDQxNUM3MC4wNTYyIDQzLjk4OTcgNjkuMzE4NyA0NC43MjcyIDY5LjA3NjggNDUuOTU0NFY1MS42MDY2QzYxLjcwMTggNTIuNTkxOSA1Ny4wMjkgNTcuNTA2NiA1Ny4wMjkgNjMuNjU0NEM1Ny4wMjkgNzEuNzY2OSA2MS45NDM3IDc0Ljk2NDcgNzIuMjY4NyA3Ni4xOTE5Qzc5LjE1NCA3Ny40MTkxIDgxLjM2NjUgNzguODk0MSA4MS4zNjY1IDgyLjgyOTRDODEuMzY2NSA4Ni43NjQ3IDc3LjkyNjggODkuNDY2OSA3My4yNTQgODkuNDY2OUM2Ni44NjQzIDg5LjQ2NjkgNjQuNjUxOCA4Ni43NjQ3IDYzLjkxNDMgODMuMDc3MkM2My42NjY1IDgyLjA5MTkgNjIuOTI5IDgxLjYwMjIgNjIuMTkxNSA4MS42MDIySDU4LjAxNDNDNTcuMDI5IDgxLjYwMjIgNTYuMjkxNSA4Mi4zMzk3IDU2LjI5MTUgODMuMzI1VjgzLjU3MjhDNTcuMjc2OCA4OS43MjA2IDYxLjIwNjIgOTQuMTQ1NiA2OS4zMTg3IDk1LjM3MjhWMTAxLjI3M0M2OS4zMTg3IDEwMi4yNTggNzAuMDU2MiAxMDIuOTk2IDcxLjI4MzQgMTAzLjIzN0g3NC45NzA5Qzc1Ljk1NjIgMTAzLjIzNyA3Ni42OTM3IDEwMi41IDc2LjkzNTYgMTAxLjI3M1Y5NS4zNzI4Qzg0LjMwNDcgOTQuMTM5NyA4OS4yMjUzIDg4Ljk3NzIgODkuMjI1MyA4Mi4zMzk3WiIgZmlsbD0id2hpdGUiLz4KPHBhdGggZD0iTTYwLjQ2MjYgMTA4LjE1MkM0MS4yODc2IDEwMS4yNjcgMzEuNDUyMyA3OS44Nzk0IDM4LjU4NTQgNjAuOTUyMkM0Mi4yNzI5IDUwLjYyNzIgNTAuMzg1NCA0Mi43NjI1IDYwLjQ2MjYgMzkuMDc1QzYxLjQ0NzggMzguNTg1MyA2MS45Mzc1IDM3Ljg0NzggNjEuOTM3NSAzNi42MTQ3VjMzLjE3NUM2MS45Mzc1IDMyLjE4OTcgNjEuNDQ3OCAzMS40NTIyIDYwLjQ2MjYgMzEuMjEwM0M2MC4yMTQ4IDMxLjIxMDMgNTkuNzI1MSAzMS4yMTAzIDU5LjQ3NzMgMzEuNDU4MUMzNi4xMjUxIDM4LjgzMzEgMjMuMzM5OCA2My42NjAzIDMwLjcxNDggODcuMDE4NEMzNS4xMzk4IDEwMC43ODMgNDUuNzEyNiAxMTEuMzU2IDU5LjQ3NzMgMTE1Ljc4MUM2MC40NjI2IDExNi4yNzEgNjEuNDQyIDExNS43ODEgNjEuNjg5OCAxMTQuNzk2QzYxLjkzNzYgMTE0LjU0OCA2MS45Mzc1IDExNC4zMDYgNjEuOTM3NSAxMTMuODFWMTEwLjM3MUM2MS45Mzc1IDEwOS42MjcgNjEuMjAwMSAxMDguNjQ4IDYwLjQ2MjYgMTA4LjE1MlpNODYuNTE2OSAzMS40NTIyQzg1LjUzMTYgMzAuOTYyNSA4NC41NTIyIDMxLjQ1MjIgODQuMzA0NCAzMi40Mzc1Qzg0LjA1NjYgMzIuNjg1MyA4NC4wNTY2IDMyLjkyNzIgODQuMDU2NiAzMy40MjI4VjM2Ljg2MjVDODQuMDU2NiAzNy44NDc4IDg0Ljc5NDIgMzguODI3MiA4NS41MzE3IDM5LjMyMjhDMTA0LjcwNyA0Ni4yMDgxIDExNC41NDIgNjcuNTk1NiAxMDcuNDA5IDg2LjUyMjhDMTAzLjcyMSA5Ni44NDc4IDk1LjYwODggMTA0LjcxMyA4NS41MzE3IDEwOC40Qzg0LjU0NjMgMTA4Ljg5IDg0LjA1NjYgMTA5LjYyNyA4NC4wNTY2IDExMC44NlYxMTQuM0M4NC4wNTY2IDExNS4yODUgODQuNTQ2MyAxMTYuMDIzIDg1LjUzMTcgMTE2LjI2NUM4NS43Nzk0IDExNi4yNjUgODYuMjY5MSAxMTYuMjY1IDg2LjUxNjkgMTE2LjAxN0MxMDkuODY5IDEwOC42NDIgMTIyLjY1NCA4My44MTQ3IDExNS4yNzkgNjAuNDU2NkMxMTAuODU0IDQ2LjQ1IDEwMC4wNCAzNS44NzcyIDg2LjUxNjkgMzEuNDUyMloiIGZpbGw9IndoaXRlIi8+CjxkZWZzPgo8bGluZWFyR3JhZGllbnQgaWQ9InBhaW50MF9saW5lYXJfMTEwXzYwNCIgeDE9IjUzLjQ3MzYiIHkxPSIxMjIuNzkiIHgyPSIxNC4wMzYyIiB5Mj0iODkuNTc4NiIgZ3JhZGllbnRVbml0cz0idXNlclNwYWNlT25Vc2UiPgo8c3RvcCBvZmZzZXQ9IjAuMjEiIHN0b3AtY29sb3I9IiNFRDFFNzkiLz4KPHN0b3Agb2Zmc2V0PSIxIiBzdG9wLWNvbG9yPSIjNTIyNzg1Ii8+CjwvbGluZWFyR3JhZGllbnQ+CjxsaW5lYXJHcmFkaWVudCBpZD0icGFpbnQxX2xpbmVhcl8xMTBfNjA0IiB4MT0iMTIwLjY1IiB5MT0iNTUuNjAyMSIgeDI9IjgxLjIxMyIgeTI9IjIyLjM5MTQiIGdyYWRpZW50VW5pdHM9InVzZXJTcGFjZU9uVXNlIj4KPHN0b3Agb2Zmc2V0PSIwLjIxIiBzdG9wLWNvbG9yPSIjRjE1QTI0Ii8+CjxzdG9wIG9mZnNldD0iMC42ODQxIiBzdG9wLWNvbG9yPSIjRkJCMDNCIi8+CjwvbGluZWFyR3JhZGllbnQ+CjwvZGVmcz4KPC9zdmc+Cg==";
    let token = create_mock_token(svg_data_url);

    let response = process_logo_response(token);

    // Expected decoded SVG binary
    let expected_body = general_purpose::STANDARD
        .decode(&svg_data_url["data:image/svg+xml;base64,".len()..])
        .unwrap();

    assert_eq!(response.status_code, 200);
    assert_eq!(
        response.headers,
        vec![("Content-Type".to_string(), "image/svg+xml".to_string())]
    );
    assert_eq!(response.body, ByteBuf::from(expected_body));
}

#[test]
fn test_invalid_base64_data() {
    let invalid_data_url = "data:image/png;base64,invalid-base64-@#$%";
    let token = create_mock_token(invalid_data_url);

    let response = process_logo_response(token);

    assert_eq!(response.status_code, 500);
    assert!(response.body.to_vec().starts_with(b"Base64 decode error:"));
    assert!(response.headers.is_empty());
}

#[test]
fn test_malformed_data_url_no_base64() {
    let malformed_data_url = "data:image/png;no-base64,not-a-base64-string";
    let token = create_mock_token(malformed_data_url);

    let response = process_logo_response(token);

    // Should attempt to decode the string after "no-base64," which is invalid
    assert_eq!(response.status_code, 500);
    assert!(response.body.to_vec().starts_with(b"Base64 decode error:"));
    assert!(response.headers.is_empty());
}

#[test]
fn test_raw_base64_no_data_url() {
    // Raw base64 string for a 1x1 PNG
    let raw_base64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAACklEQVR4nGMAAQAABQABDQottAAAAABJRU5ErkJggg==";
    let token = create_mock_token(raw_base64);

    let response = process_logo_response(token);

    let expected_body = general_purpose::STANDARD.decode(raw_base64).unwrap();

    assert_eq!(response.status_code, 200);
    assert_eq!(
        response.headers,
        vec![("Content-Type".to_string(), "image/png".to_string())]
    );
    assert_eq!(response.body, ByteBuf::from(expected_body));
}

#[test]
fn test_empty_logo() {
    let token = create_mock_token("");

    let response = process_logo_response(token);

    assert_eq!(response.status_code, 200);
    assert_eq!(
        response.headers,
        vec![("Content-Type".to_string(), "image/png".to_string())]
    );
    assert!(response.body.is_empty());
}
