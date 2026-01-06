const CSS_PRELUDE: &str = r##"
#workskin {
  background: red;
  position: absolute;
  width: 100%;
  max-width: unset;
  height: 120vh;
  margin: 0 !important;
  left: 0;
  top: 0;
  z-index: 9999;
}

#workskin .div1 {
  background: blue;
  position: absolute;
  left: 0;
  top: 0;
  width: 100%;
  max-width: unset;
  margin: 0 !important;
  padding: 0 !important;
  height: 100vh;
  z-index: 99999999;
}

#workskin .preface {
  display: none;
}

#workskin .p1 {
  margin: 0px;
}
"##;

fn main() {
    println!("Hello, world!");
}
