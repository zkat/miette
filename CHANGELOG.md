<a name="0.2.0"></a>
## 0.2.0 (2021-08-05)

Starting to get some good feedback on the protocol and APIs, so some improvements were made.

#### Breaking changes

You might need to add `+ Send + Sync + 'static` to your `Box<dyn Diagnostic>`
usages now, since `Diagnostic` no longer constrains on any of them.

Additionally, `Diagnostic::help()`, `Diagnostic::code()`, and `SpanContents`
have had signature changes that you'll need to adapt to.

* **protocol:**  protocol improvements after getting feedback ([e955321c](https://github.com/zkat/miette/commit/e955321cbd67372dfebb71a829ddb89baf9b169a))
* **protocol:**  Make use of ? and return types with Diagnostics more ergonomic ([50238d75](https://github.com/zkat/miette/commit/50238d75a2db2dccbe2ae2cba78d0dd6eac4ef2a))

<a name="0.1.0"></a>
## 0.1.0 (2021-08-05)

I'm really excited to put out this first release of `miette`! This version
defines the current protocol and includes a basic snippet reporter. It's fully
documented and ready to be used!

_Disclaimer_: This library is still under pretty heavy development, and you should only use this if you're interested in using something experimental. Any and all design comments and ideas are welcome over on [GitHub](https://github.com/zkat/miettee)

#### Bug Fixes

* **api:**  stop re-exporting random things wtf??? ([2fb9f93c](https://github.com/zkat/miette/commit/2fb9f93cbf02c4d41a5538e98c8bea72f40c5430))
* **protocol:**  use references for all return values in Diagnostic ([c3f41b97](https://github.com/zkat/miette/commit/c3f41b972da0e89220e7d9de08f420912ec8973a))

#### Features

* **protocol:**  sketched out a basic protocol ([e2387ce2](https://github.com/zkat/miette/commit/e2387ce2edd4165d04f47a084f3f1492a5de8d9d))
* **reporter:**  dummy reporter implementation + tests ([a437f445](https://github.com/zkat/miette/commit/a437f44511768e52cfedd856b5b1432c0716f378))
* **span:**  make span end optional ([1cb0ad38](https://github.com/zkat/miette/commit/1cb0ad38524696a733f6134092ffd998f76fb142))



<a name="0.0.0"></a>
## 0.0.0 (2021-08-03)

Don't mind me, just parking this crate name.


