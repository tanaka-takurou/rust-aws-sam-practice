use lambda_http::{run, service_fn, Error, IntoResponse, Request, Response};
use tera::{Context, Tera};

static SIMPLE_TEMPLATE: &'static str = "<!doctype html>
<html>
  <head>
    <meta charset='utf-8'>
    <meta http-equiv='X-UA-Compatible' content='IE=edge,chrome=1'>
    <meta name='viewport' content='width=device-width, initial-scale=1.0, maximum-scale=1.0'>
    <title>Sample Page</title>
    <script src='https://code.jquery.com/jquery-3.4.1.min.js' integrity='sha256-CSXorXvZcTkaix6Yvo6HppcZGetbYMGWSFlBw8HfCJo=' crossorigin='anonymous'></script>
    <script src='https://cdnjs.cloudflare.com/ajax/libs/semantic-ui/2.4.1/semantic.min.js'></script>
    <link rel='stylesheet' href='https://cdnjs.cloudflare.com/ajax/libs/semantic-ui/2.4.1/semantic.min.css'>
    <style type='text/css'>
.last.container {
  margin-bottom: 300px !important;
}
h1.ui.center.header {
  margin-top: 3em;
}
h2.ui.center.header {
  margin: 4em 0em 2em;
}
h3.ui.center.header {
  margin-top: 2em;
  padding: 2em 0em;
}
.ui.footer.segment {
  margin: 5em 0em 0em;
  padding: 5em 0em;
}
.ui.container.vmargin {
  margin: 5rem 0em calc(2rem - 0.14285714em );
}
    </style>
  </head>
  <body>
    <div class='ui fixed inverted menu'>
      <div class='ui container'>
        <a href='#' class='item'>Home</a>
      </div>
    </div>
    <h1 class='ui center aligned header'>Sample Page</h1>
    <div class='ui container vmargin'>
      <div class='ui relaxed divided items'>
{% for item in items %}
        <div class='item'>
          <div class='ui small image'>
            <img src='data:image/jpeg;base64,/9j/4AAQSkZJRgABAQAAAQABAAD/2wBDAA8PDw8PDw8PDw8PDw8PDw8PDw8PDw8PDw8PDw8PDw8PDw8PDw8PDw8PDw8PDw8PDw8PDw8PDw8PDw8PDw8RERH/wQARCAB6AJQDABEAAREAAhEA/8QASQABAQEBAQEBAAAAAAAAAAAAAAQDAgUBBhABAQAAAwUGBQMFAQAAAAAAAAECAxEEITFScRITFDNBwSQygpHwUWGBIiNyobHR/9oADAMAAAEAAgAAPwD9C+vgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADbIkxZmlms0t00/TQFly8qccOGAd3lcuEDu8rlwgd3lcuEDu8rlwgd3lcuGgzzsvLmXbMOGWab/wCQQgAAAAAAF4ATgAAN9n8z6b7A72r5sPS+wJQAPS7wVYNn7WGW2y2btOF6gnsuHFcN4y6AuzvKvSA88AAAAAAAAAAG+z+Z9N9gd7V82HoCaS2yTjboC6ZE7u4LNMV36/v6AjmDFcVwyW2a6yekgNcO0YsGHs6TWTSW8YDHW3FreNsv+wXZ3lXpAeeAAAAAAAAAADfZ/M+m+wO9q+bD0/8AAdbPl2f13jv09/YFWvACSS2ySW8bONBLn5WuuPDOoJJxnWf9BfneVekB54AAAAAAAAAAN9nn9yf432Btn5eLHcNwzXSXXfN2t3celBxJtEkkmknDfhB9+J/OyB8T+dkD4n87IM5k5tutw79ZbdZPX9gVZ00yrNNNJAecAAAAAAAAAADTKxzLxdrS3dZpr+oKPFTks/kDxU5b9wPFTlv3A8VOW/cDxU5b9wPFTlv3BxmZ8x4bhmGzX111BMAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD/2Q=='>
          </div>
          <div class='content'>
            <a class='header'>Header {{ item }}</a>
            <div class='meta'>
              <a>Date {{ item }}</a>
              <a>Category {{ item }}</a>
            </div>
            <div class='description'>
              A description which may flow for several lines and give context to the content.
            </div>
          </div>
        </div>
{% endfor %}
      </div>
    </div>
    <div class='ui inverted vertical footer segment'>
      <div class='ui center aligned container'>
        <div class='ui horizontal inverted small divided link list'>
          <a class='item' href='#'>Contact</a>
          <a class='item' href='#'>Privacy Policy</a>
        </div>
      </div>
    </div>
  </body>
</html>";

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/lambda-http/examples
async fn function_handler(_event: Request) -> Result<impl IntoResponse, Error> {
    let sample_vec = vec![1, 2, 3];

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![("sample.html", SIMPLE_TEMPLATE)]).unwrap();
    let mut ctx = Context::new();
    ctx.insert("items", &sample_vec);

    let html = tera.render("sample.html", &ctx).unwrap();

    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(html)
        .map_err(Box::new)?;
    Ok(resp)
}



#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
