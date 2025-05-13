use std::io::Write;

pub struct Attributes<'a>(Vec<(&'a str, &'a str)>);

impl<'a> Attributes<'a> {
    pub fn new(pairs: Vec<(&'a str, &'a str)>) -> Self {
        Self(pairs)
    }

    pub fn write_to<W: Write>(
        &self,
        writer: &mut W,
    ) -> std::io::Result<()> {
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
        Self {
            writer,
            indent_level: 0,
            indent_spaces: 2,
        }
    }

    fn write_indent(&mut self) -> std::io::Result<()> {
        for _ in 0..(self.indent_level * self.indent_spaces) {
            write!(self.writer, " ")?;
        }
        Ok(())
    }

    pub fn open_tag(
        &mut self,
        tag: &str,
        attrs: Option<Attributes>,
        //attrs: Option<&[(&str, &str)]>,
    ) -> std::io::Result<()> {
        self.write_indent()?;
        write!(self.writer, "<{}", tag)?;
        if let Some(attrs) = attrs {
            attrs.write_to(&mut self.writer)?;
        }
        writeln!(self.writer, ">")?;
        self.indent_level += 1;
        Ok(())
    }

    pub fn close_tag(
        &mut self,
        tag: &str,
    ) -> std::io::Result<()> {
        self.indent_level -= 1;
        self.write_indent()?;
        writeln!(self.writer, "</{}>", tag)
    }

    pub fn self_closing_tag(
        &mut self,
        tag: &str,
        attrs: Option<Attributes>,
    ) -> std::io::Result<()> {
        self.write_indent()?;
        write!(self.writer, "<{}", tag)?;
        if let Some(attrs) = attrs {
            attrs.write_to(&mut self.writer)?;
        }
        writeln!(self.writer, " />")
    }

    pub fn text_element(
        &mut self,
        tag: &str,
        text: &str,
    ) -> std::io::Result<()> {
        self.write_indent()?;
        writeln!(self.writer, "<{tag}>{text}</{tag}>")
    }

    pub fn text_element_with_attrs(
        &mut self,
        tag: &str,
        text: &str,
        attrs: Attributes,
    ) -> std::io::Result<()> {
        self.write_indent()?;
        writeln!(self.writer, "<{tag}{}>{text}</{tag}>", attrs.to_string())
    }

    pub fn raw(
        &mut self,
        s: &str,
    ) -> std::io::Result<()> {
        writeln!(self.writer, "{}", s)
    }
}
