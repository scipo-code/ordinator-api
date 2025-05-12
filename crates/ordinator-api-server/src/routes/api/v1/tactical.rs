            let orchestrator = orchestrator.lock().unwrap();

            Ok(orchestrator
                .handle_tactical_request(tactical_request)
                .await?)
