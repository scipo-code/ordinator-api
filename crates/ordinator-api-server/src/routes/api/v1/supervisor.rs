            let orchestrator = orchestrator.lock().unwrap();

            Ok(orchestrator
                .handle_supervisor_request(supervisor_request)
                .await?)
