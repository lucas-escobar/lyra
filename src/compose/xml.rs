/// Write formatted XML to any writable
use std::io::Write;

type Result = std::io::Result<()>;

pub struct XmlAttributes<'a>(Vec<(&'a str, &'a str)>);

impl<'a> XmlAttributes<'a> {
    pub fn new(pairs: Vec<(&'a str, &'a str)>) -> Self {
        Self(pairs)
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result {
        for (k, v) in &self.0 {
            write!(writer, " {}=\"{}\"", k, v)?;
        }
        Ok(())
    }

    pub fn to_string(&self) -> String {
        self.0
            .iter()
            .map(|(k, v)| format!(" {}=\"{}\"", k, v))
            .collect::<Vec<_>>()
            .join("")
    }
}

pub struct Writer<W: Write> {
    writer: W,
    indent_level: usize,
    indent_spaces: usize,
}

impl<W: Write> Writer<W> {
    pub fn new(writer: W) -> Self {
        Self { writer, indent_level: 0, indent_spaces: 2 }
    }

    fn write_indent(&mut self) -> Result {
        for _ in 0..(self.indent_level * self.indent_spaces) {
            write!(self.writer, " ")?;
        }
        Ok(())
    }

    pub fn open_tag(
        &mut self,
        tag: &str,
        attrs: Option<XmlAttributes>,
    ) -> Result {
        self.write_indent()?;
        write!(self.writer, "<{}", tag)?;
        if let Some(attrs) = attrs {
            attrs.write_to(&mut self.writer)?;
        }
        writeln!(self.writer, ">")?;
        self.indent_level += 1;
        Ok(())
    }

    pub fn close_tag(&mut self, tag: &str) -> Result {
        self.indent_level -= 1;
        self.write_indent()?;
        writeln!(self.writer, "</{}>", tag)
    }

    pub fn self_closing_tag(
        &mut self,
        tag: &str,
        attrs: Option<XmlAttributes>,
    ) -> Result {
        self.write_indent()?;
        write!(self.writer, "<{}", tag)?;
        if let Some(attrs) = attrs {
            attrs.write_to(&mut self.writer)?;
        }
        writeln!(self.writer, " />")
    }

    pub fn text_element(&mut self, tag: &str, text: &str) -> Result {
        self.write_indent()?;
        writeln!(self.writer, "<{tag}>{text}</{tag}>")
    }

    //TODO combine this with text_element()
    pub fn text_element_with_attrs(
        &mut self,
        tag: &str,
        text: &str,
        attrs: XmlAttributes,
    ) -> Result {
        self.write_indent()?;
        writeln!(self.writer, "<{tag}{}>{text}</{tag}>", attrs.to_string())
    }

    /// Write raw string (non XML) to internal writer
    pub fn raw(&mut self, s: &str) -> Result {
        writeln!(self.writer, "{}", s)
    }
}
