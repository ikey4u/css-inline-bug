# css-inline-bug

- Problem

    Requesting remote css resource in axum router slows down the response

- Reproduce

    Run `cargo run`, open your browser and access `http://127.0.0.1:3009` in
    a new tab (without any cache enabled).

    And you will see the request costs a few seconds to finish, and this demo
    will give the following output

        failed to inline css style: Network(Error(Io(Os { code: 35, kind: WouldBlock, message: "Resource temporarily unavailable" })))


