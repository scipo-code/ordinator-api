fn temp_wrapper() {

    // let mut priority_queues = PriorityQueues::<u32, u32> {
    //     unloading: PriorityQueue::new(),
    //     shutdown_vendor: PriorityQueue::new(),
    //     normal: PriorityQueue::new(),
    // };
    
    // populate_priority_queues(&self.backlog, &mut priority_queues);
    
    // loop {
    //     println!("I am the scheduler agent and I received a message");
    //     println!("Hello I am scheduling");
    //     sleep(Duration::from_secs(1));
    // }
}


struct PriorityQueues<T, P> 
    where T: Hash + Eq,
          P: Ord
{ 
    unloading: PriorityQueue<T, P>,
    shutdown_vendor: PriorityQueue<T, P>,
    normal: PriorityQueue<T, P>,
}

fn populate_priority_queues(backlog: &Vec<WorkOrder>, priority_queues: &mut PriorityQueues<u32, u32>) -> () {
    for work_order in backlog {
        if work_order.unloading_point.present {
            priority_queues.unloading.push(work_order.order_number, work_order.order_weight);
        } else if work_order.revision.shutdown || work_order.vendor {
            priority_queues.shutdown_vendor.push(work_order.order_number, work_order.order_weight);
        } else {
            priority_queues.normal.push(work_order.order_number, work_order.order_weight);
        }
    }
}

