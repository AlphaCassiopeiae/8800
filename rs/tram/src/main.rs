// TO BE USED BY CYCLE

// state machine to handle CPU reading from memory
// returns data
async fn cpu_mem_read(addr) {
    // CHECK IN RUN MODE

    // T1
    // place addr on bus
    // set smemr to 1

    // wait clk 1

    // set pdbin low

    // wait for xrdy (should go low on T2)

    // wait one clk cycle (Tw)

    // if / wait xrdy to go high

    // data available
    // deassert smemr and pdbin

    return data;
}


// state machine to handle CPU writing to memory
async fn cpu_mem_write(addr, data) {


    
}


// TO BE USED BY RECALL
async fn mem_cpu_read() {

}

fn mem_cpu_write() {

}