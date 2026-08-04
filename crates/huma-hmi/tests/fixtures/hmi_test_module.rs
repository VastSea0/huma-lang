use huma_hmi::{
    read_frame, write_frame, HmiValue, ProtocolVersion, Request, RequestOperation, Response,
    ResponsePayload,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut input = stdin.lock();
    let mut output = stdout.lock();

    loop {
        let request = read_frame::<Request>(&mut input)?;
        request.validate()?;
        let mut request_id = request.request_id;
        let (response, stop) = match request.operation {
            RequestOperation::Initialize { .. } => {
                (ResponsePayload::Success(HmiValue::Empty), false)
            }
            RequestOperation::Call {
                function,
                arguments,
            } if function == "topla" => {
                let total = arguments
                    .into_iter()
                    .map(|value| match value {
                        HmiValue::Number(number) => Ok(number),
                        _ => Err("topla yalnız sayı kabul eder"),
                    })
                    .sum::<Result<f64, _>>()?;
                (ResponsePayload::Success(HmiValue::Number(total)), false)
            }
            RequestOperation::Call { function, .. } if function == "yanlis_id" => {
                request_id += 1;
                (ResponsePayload::Success(HmiValue::Empty), false)
            }
            RequestOperation::Call { function, .. } if function == "bekle" => {
                std::thread::sleep(std::time::Duration::from_secs(30));
                (ResponsePayload::Success(HmiValue::Empty), false)
            }
            RequestOperation::Call { function, .. } => (
                ResponsePayload::Failure(huma_hmi::RemoteError {
                    code: "TEST-BULUNAMADI-1".to_string(),
                    message: format!("fonksiyon bulunamadı: {function}"),
                    retryable: false,
                }),
                false,
            ),
            RequestOperation::Shutdown => (ResponsePayload::Success(HmiValue::Empty), true),
        };
        write_frame(
            &mut output,
            &Response {
                protocol: ProtocolVersion::V1_0,
                request_id,
                response,
            },
        )?;
        if stop {
            break;
        }
    }
    Ok(())
}
