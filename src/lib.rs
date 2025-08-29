mod pipeline;
mod old_lib;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_pipeline() {
        let step1 = || { println!("func 1"); Ok(()) };
        let step2 = || {
            let result = add(4, 5);
            println!("func 2 -> {result}");
            Ok(())
        };


        pipeline::Pipeline::new().
            step("func 1", step1).
            step("func 2", step2).
            run().
            expect("step failed");

        // assert_eq!(res., true);
    }
}
