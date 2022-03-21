## stratum_ping

>**Measuring round-trip delay time (RTD) using Stratum protocol**

#### 🤔 About

A mining-specific approach when we measure the amount of time required to connect and successfully pass authentication using the Stratum protocol.

#### 🚀 Usage

Here are a few brief examples to get you started

```
./stratum_ping  --server ethash.poolbinance.com:3333 --proto "stratum2"
```

or 

``` 
./stratum_ping  --server ethash.poolbinance.com:3333 --proto "stratum1" --w 10 --login admin --pass 1234
```
