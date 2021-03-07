/* Examples for testing */

true;
if false then true else false; 

0; 
succ (pred 0);
iszero (pred (succ (succ 0))); 


/* if 0 then 1 else pred 1; */
if iszero (pred (succ 0)) then succ 0 else pred (succ 0); 