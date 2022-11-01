use crate::traits::NumOps;

#[derive(Copy, Clone, Debug)]
pub enum ThresholdMethod
{
    Binary,
    BinaryInv,
    ThreshTrunc,
    ThreshToZero,
}
impl ThresholdMethod
{
    pub fn from_string_result(input: &str) -> Result<Self, String>
    {
        match input
        {
            "binary" => Ok(Self::Binary),
            "binary_inv"=>Ok(Self::BinaryInv),
            "thresh_trunk"=>Ok(Self::ThreshTrunc),
            "thresh_to_zero"=>Ok(Self::ThreshToZero),
            _ => Err("Unknown threshold type,accepted values are binary,binary_inv,thresh_trunc,thresh_to_zero".to_string()),
        }
    }
}
#[rustfmt::skip]
pub fn threshold<T:NumOps<T>+Copy+Ord+PartialOrd>(in_image: &mut [T], threshold: T, method: ThresholdMethod)
{
    let max = T::max_val();
    let min = T::min_val();
    match method
    {
        ThresholdMethod::Binary => {
            for x in in_image.iter_mut()
            {
                *x = { 
                    
                    if *x > threshold {
                        max
                    } else {
                        min
                    }
                }
            }
        }
        ThresholdMethod::BinaryInv => {
            for x in in_image.iter_mut()
            {
                *x = {
                    if *x > threshold {
                        min
                    } else {
                        max
                    }
                }
            }
        }
        ThresholdMethod::ThreshTrunc => {
            for x in in_image.iter_mut()
            {
                *x = {
                    if *x > threshold {
                        threshold
                    } else {
                        *x
                    }
                }
            }
        }
        ThresholdMethod::ThreshToZero => {
            for x in in_image.iter_mut()
            {
                *x = {
                    if *x > threshold {
                        threshold
                    } else {
                        T::min_val()
                    }
                }
            }
        }
    }
}

#[cfg(all(feature = "benchmarks"))]
#[cfg(test)]
mod benchmarks
{
    extern crate test;

    #[bench]
    fn threshold_scalar(b: &mut test::Bencher)
    {
        use crate::threshold::threshold;

        let width = 800;
        let height = 800;
        let dimensions = width * height;

        let mut c1 = vec![0; dimensions];

        b.iter(|| {
            threshold(&mut c1, 10, crate::threshold::ThresholdMethod::BinaryInv);
        });
    }
}
