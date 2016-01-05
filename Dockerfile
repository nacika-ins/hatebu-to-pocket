FROM jimmycuadra/rust
ADD . /source
RUN cargo build --release
CMD ["/source/target/release/hatebu-to-pocket"]

