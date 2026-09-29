


#[cfg(test)]
mod tests {

    #[test]
    fn test_print() -> Result<(), String> {
        let x = 4.0;
        println!("this value is {}", x);
        Ok(())
    }
}