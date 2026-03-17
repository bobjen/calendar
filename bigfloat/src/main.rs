use bigfloat::BigFloat;

fn main() {
    let mut m: Vec<Vec<BigFloat>> = vec![
        vec![BigFloat::new(), BigFloat::from_int(1), BigFloat::from_int(1)],
        vec![BigFloat::from_int(1), BigFloat::from_int(1), BigFloat::from_int(2)],
    ];

    BigFloat::gaussian_elimination(&mut m, 2, 2);
    println!("{}, {}", m[0][2].to_integer() as i32, m[1][2].to_integer() as i32);

    println!("{}", BigFloat::pi().to_double());

    #[cfg(feature = "bigfloat-test")]
    BigFloat::unit_test();
}
