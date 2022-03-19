## stratum_ping

>**The measuring round-trip delay time (RTD) using Stratum protocol**

#### 🤔 About

RTD is defined as a metric that measures in milliseconds the amount of time it takes for a data packet to be sent plus the amount of time it takes for acknowledgement of that signal to be received.


#### 💿 Installation and Quick Start

#### 🚀 Usage

Here are a few brief examples to get you started

` 
./stratum_ping  --server ethash.poolbinance.com:3333 --proto "stratum2"
`
or 
` 
./stratum_ping  --server ethash.poolbinance.com:3333 --proto "stratum1" --attempts 10 --login admin --pass 1234
`
