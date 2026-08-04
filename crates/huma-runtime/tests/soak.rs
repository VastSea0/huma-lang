use huma_runtime::gc::{collect_cycles, Gc};
use huma_runtime::isolate::{Isolate, IsolateConfig, IsolateRequest, IsolateValue};
use huma_runtime::value::Deger;

#[test]
#[ignore = "scheduled soak gate"]
fn elli_bin_dongusel_tahsis_heap_kaydini_buyutmez() {
    for index in 0..50_000 {
        let list = Gc::new(Vec::new());
        list.borrow_mut().push(Deger::Liste(list.clone()));
        drop(list);
        if index % 64 == 63 {
            let stats = collect_cycles();
            assert!(stats.examined <= 64, "heap kaydı büyüdü: {stats:?}");
            assert_eq!(stats.reclaimed_cycles, stats.examined, "{stats:?}");
        }
    }
    collect_cycles();
    let settled = collect_cycles();
    assert_eq!(settled.examined, 0, "heap platosuna inmedi: {settled:?}");
}

#[test]
#[ignore = "scheduled soak gate"]
fn isolate_heapleri_paralel_ve_bagimsiz_kalir() {
    let workers = (0..8)
        .map(|worker| {
            std::thread::spawn(move || {
                let isolate = Isolate::spawn(IsolateConfig::default()).unwrap();
                isolate
                    .execute(IsolateRequest::new("sayac = 0 olsun"))
                    .unwrap();
                for expected in 1..=250 {
                    let mut request = IsolateRequest::new("sayac = sayac + 1 olsun");
                    request.exports.push("sayac".to_string());
                    let response = isolate.execute(request).unwrap();
                    assert_eq!(
                        response.exports.get("sayac"),
                        Some(&IsolateValue::Number(expected as f64)),
                        "isolate {worker} başka heap ile karıştı"
                    );
                }
            })
        })
        .collect::<Vec<_>>();
    for worker in workers {
        worker.join().expect("soak worker paniklememeli");
    }
}
