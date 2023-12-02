//! CAS login service for XIDIAN University
//!
//! ## Auth Flow
//!
//! 1. User clicks the login button on the client side.
//! 2. Client redirects user to the server, the address is
//!    `https://ids.xidian.edu.cn/authserver/login?service=`,
//!    the service query param stand for the callback url.
//! 3. User login on the CAS login page.
//! 4. CAS redirects user to the callback url with a ticket.
//!
