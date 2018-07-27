service-pre
===========

Service traits and implementation (formerly part of mydht-base).

Those are service as a process (function call, coroutine, thread ...), that read on a channel and reply on another channel.  
The services are preemptible (can suspend and resume : particalarily on input channel, but also on other elements (asynch transport or others).  
A suspended service should not restart by itself, but depending on implementation spurious restore could happen.  
Similarily restore could be call without checking the service state and should be costless (if not a state should be checked).  

Status
------

WIP

