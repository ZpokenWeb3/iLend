#[cfg(test)]
mod tests {
    use master_contract::contract::DecimalExt;
    use rust_decimal::Decimal;

    #[test]
    fn test_rescale_with_return() {
        let scale_decimal_one = 18;
        let scale_decimal_two = 6;

        let first_amount = 50 * 10u128.pow(scale_decimal_one);
        let second_amount = 50 * 10u128.pow(scale_decimal_two);

        let decimal_one = Decimal::from_i128_with_scale(first_amount as i128, scale_decimal_one);
        let decimal_two = Decimal::from_i128_with_scale(second_amount as i128, scale_decimal_two)
            .rescale_with_return(scale_decimal_one);

        let _decimal_third =
            Decimal::from_i128_with_scale(second_amount as i128, scale_decimal_one)
                .rescale_with_return(scale_decimal_one);

        // decimal_third is converting NOT as expected in sequence of applying
        // from_i128_with_scale ( amount, 18) .rescale_with_return(18)

        let decimal_one_zero = Decimal::from_i128_with_scale(0 as i128, scale_decimal_one);
        let decimal_two_zero = Decimal::from_i128_with_scale(0 as i128, scale_decimal_two)
            .rescale_with_return(scale_decimal_one);

        assert_eq!(
            decimal_one_zero.to_u128_with_decimals().unwrap(),
            decimal_two_zero.to_u128_with_decimals().unwrap()
        );

        assert_eq!(
            decimal_one.to_u128_with_decimals().unwrap(),
            decimal_two.to_u128_with_decimals().unwrap()
        );
    }
}
