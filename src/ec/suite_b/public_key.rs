// Copyright 2016 Brian Smith.
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHORS DISCLAIM ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY
// SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION
// OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
// CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

//! Functionality shared by operations on public keys (ECDSA verification and
//! ECDH agreement).

use super::ops::*;
use untrusted;

/// Parses a public key encoded in uncompressed form. The key's coordinates are
/// verified to be valid field elements and the point is verified to be on the
/// curve. (The point cannot be at infinity because it is given in affine
/// coordinates.)
pub fn parse_uncompressed_point<'a>(ops: &PublicKeyOps,
                                    input: untrusted::Input<'a>)
                                    -> Result<(Elem, Elem), ()> {
    let (x, y) = try!(input.read_all((), |input| {
        // The encoding must be 4, which is the encoding for "uncompressed".
        let encoding = try!(input.read_byte());
        if encoding != 4 {
            return Err(());
        }
        let x = try!(ops.elem_parse(input));
        let y = try!(ops.elem_parse(input));
        Ok((x, y))
    }));

    // Verify that (x, y) is on the curve, which is true iif:
    //
    //     y**2 == x**3 + a*x + b
    //
    // Or, equivalently, but more efficiently:
    //
    //     y**2 == (x**2 + a)*x + b

    let lhs = ops.common.elem_sqr(&y);

    let mut rhs = ops.common.elem_sqr(&x);
    ops.elem_add(&mut rhs, &ops.a);
    let mut rhs = ops.common.elem_mul(&rhs, &x);
    ops.elem_add(&mut rhs, &ops.b);
    if !ops.elems_are_equal(&lhs, &rhs) {
        return Err(());
    }

    Ok((x, y))
}


#[cfg(test)]
mod tests {
    use file_test;
    use super::*;
    use super::super::ops;
    use untrusted;

    #[test]
    fn parse_uncompressed_point_test() {
         file_test::run("src/ec/suite_b/suite_b_public_key_tests.txt",
                        |section, test_case| {
            assert_eq!(section, "");

            let curve_name = test_case.consume_string("Curve");
            let curve_ops = if curve_name == "P-256" {
                &ops::p256::PUBLIC_KEY_OPS
            } else if curve_name == "P-384" {
                &ops::p384::PUBLIC_KEY_OPS
            } else {
                panic!("Unsupported curve: {}", curve_name);
            };

            let public_key = test_case.consume_bytes("Q");
            let public_key = try!(untrusted::Input::new(&public_key));
            let valid = test_case.consume_string("Result") == "P";

            let result = parse_uncompressed_point(curve_ops, public_key);
            assert_eq!(valid, result.is_ok());

            // TODO: Verify that we when we re-serialize the parsed (x, y), the
            // output is equal to the input.

            Ok(())
        });
    }
}