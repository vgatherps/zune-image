use zune_core::bit_depth::{BitDepth, BitType};
use zune_core::colorspace::ColorSpace;

use crate::codecs::ImageFormat;
use crate::errors::{ImgEncodeErrors, ImgErrors, ImgOperationsErrors};
use crate::image::Image;
use crate::metadata::ImageMetadata;
use crate::workflow::EncodeResult;

/// Encapsulates an image decoder.
///
/// All supported image decoders must implement this class
pub trait DecoderTrait<'a>
{
    /// Decode a buffer already in memory
    ///
    /// The buffer to be decoded is the one passed
    /// to the decoder when initializing the decoder
    ///
    /// # Returns
    /// - Image -> Pixels decoded from the image as interleaved pixels.
    ///
    /// # Errors
    ///  - Any image decoding errors will be propagated to the caller.
    ///
    /// # Example
    /// ```no_run
    /// use zune_image::traits::DecoderTrait;
    /// use zune_jpeg::JpegDecoder;
    /// let mut decoder = JpegDecoder::new(&[0xFF,0xD8]);
    ///
    /// decoder.decode().unwrap();
    /// ```
    fn decode(&mut self) -> Result<Image, crate::errors::ImgErrors>;

    /// Get width and height of the image
    ///
    /// # Returns
    /// - Some(width,height)
    /// - None -> If image hasn't been decoded and we can't extract
    ///  the width and height.
    fn get_dimensions(&self) -> Option<(usize, usize)>;

    /// Get the colorspace that the decoded pixels
    /// are stored in.
    fn get_out_colorspace(&self) -> ColorSpace;

    /// Get the name of the decoder
    fn get_name(&self) -> &'static str;

    /// Return true whether or not this codec is fully supported
    /// and well tested to handle various formats.
    ///
    /// Currently set to true but a codec that is experimental should override it
    /// to be false
    fn is_experimental(&self) -> bool
    {
        false
    }
    /// Read image metadata returning the values as
    /// a struct
    fn read_headers(&mut self) -> Result<Option<ImageMetadata>, crate::errors::ImgErrors>
    {
        Ok(None)
    }
}

/// This encapsulates an image operation.
///
/// All operations that can be stored in a workflow
/// need to encapsulate this struct.
pub trait OperationsTrait
{
    /// Get the name of this operation
    fn get_name(&self) -> &'static str;

    /// Execute a simple operation on the image
    /// manipulating the image struct
    ///
    /// An object should implement this function, but
    /// a caller should call [`execute`], which does some error checking
    /// before calling this method
    ///
    /// [`execute`]: Self::execute
    fn execute_impl(&self, image: &mut Image) -> Result<(), ImgOperationsErrors>;

    /// Return the supported colorspaces this operation supports
    ///
    /// Some operations cannot work on all colorspaces, e.g rgb to grayscale will
    /// only work on RGB colorspace, not YUV or YCbCr colorspace, hence such an operation
    /// must only declare support for such colorspace
    ///
    /// During execution, the image colorspace will be matched to this colorspace and
    /// if it doesn't support it, an error will be raised during execution
    fn supported_colorspaces(&self) -> &'static [ColorSpace]
    {
        &[
            ColorSpace::RGBA,
            ColorSpace::RGB,
            ColorSpace::LumaA,
            ColorSpace::Luma,
            ColorSpace::CMYK,
            ColorSpace::YCbCr,
            ColorSpace::YCCK
        ]
    }
    /// Get supported bit types for this operation
    ///
    /// Not all operations are supported for all bit types and
    /// o each support requires careful analysis to ensure it's doing
    /// the right things
    fn supported_types(&self) -> &'static [BitType];

    /// Execute an operation
    ///
    /// This does come common error checking operations, e.g
    /// it checks that image dimensions match array length and that this operation
    /// supports the image colorspace, before calling [`execute_impl`]
    ///
    /// # Arguments
    /// - image: A mutable reference to an image which
    /// this operation will manipulate
    ///
    ///
    /// # Errors
    /// Any operations error will be propagated to the caller
    ///
    ///
    /// [`execute_impl`]: Self::execute_impl
    fn execute(&self, image: &mut Image) -> Result<(), ImgErrors>
    {
        // Confirm colorspace
        let colorspace = image.get_colorspace();

        let supported = self
            .supported_colorspaces()
            .iter()
            .any(|x| *x == colorspace);

        if !supported
        {
            return Err(ImgErrors::UnsupportedColorspace(
                colorspace,
                self.get_name(),
                self.supported_colorspaces()
            ));
        }
        // check we support the bit depth
        let bit_type = image.metadata.get_depth().bit_type();

        let supported = self.supported_types().iter().any(|x| *x == bit_type);

        if !supported
        {
            return Err(ImgOperationsErrors::UnsupportedType(self.get_name(), bit_type).into());
        }

        confirm_invariants(image)?;

        self.execute_impl(image)
            .map_err(<ImgOperationsErrors as Into<ImgErrors>>::into)?;

        confirm_invariants(image)?;

        Ok(())
    }
}

/// Confirm that image invariants have been respected across image
/// operations
fn confirm_invariants(image: &Image) -> Result<(), ImgErrors>
{
    // Ensure dimensions are correct

    let components = image.get_channels_ref(false).len();

    if components != image.get_colorspace().num_components()
    {
        return Err(ImgErrors::GenericString(format!(
            "Components mismatch, expected {} channels since image format is {:?}, but found {}",
            image.get_colorspace().num_components(),
            image.get_colorspace(),
            components
        )));
    }
    let (width, height) = image.get_dimensions();
    // check the number of channels match the length

    let expected_length = image.get_depth().size_of() * width * height;

    for channel in image.get_channels_ref(true)
    {
        if channel.len() != expected_length
        {
            return Err(ImgErrors::DimensionsMisMatch(
                expected_length,
                channel.len()
            ));
        }
    }

    Ok(())
}

pub trait EncoderTrait
{
    /// Get the name of the encoder
    fn get_name(&self) -> &'static str;

    /// Encode and write to a file
    ///
    /// The file is stored internally by the decoder, e.g
    /// by asking for it during initialization
    ///
    /// # Arguments
    /// - image: An image which we are trying to encode.
    ///
    fn encode_inner(&mut self, image: &Image) -> Result<Vec<u8>, ImgEncodeErrors>;

    /// Return all colorspaces supported by this encoder.
    ///
    /// An encoder should reject any other colorspace and should not try to write
    /// an unknown colorspace
    fn supported_colorspaces(&self) -> &'static [ColorSpace];

    fn encode(&mut self, image: &Image) -> Result<Vec<u8>, ImgEncodeErrors>
    {
        // check colorspace is correct.
        let colorspace = image.get_colorspace();
        let supported_colorspaces = self.supported_colorspaces();

        if !supported_colorspaces.contains(&colorspace)
        {
            return Err(ImgEncodeErrors::UnsupportedColorspace(
                colorspace,
                supported_colorspaces
            ));
        }

        self.encode_inner(image)
    }
    fn format(&self) -> ImageFormat;

    fn encode_to_result(&mut self, image: &Image) -> Result<EncodeResult, ImgEncodeErrors>
    {
        let data = self.encode(image)?;

        Ok(EncodeResult {
            data,
            format: self.format()
        })
    }
}

pub trait ZuneInts<T>
{
    fn depth() -> BitDepth;
}

impl ZuneInts<u8> for u8
{
    #[inline(always)]
    fn depth() -> BitDepth
    {
        BitDepth::Eight
    }
}

impl ZuneInts<u16> for u16
{
    #[inline(always)]
    fn depth() -> BitDepth
    {
        BitDepth::Sixteen
    }
}
