


label main_entry(define cla) &[visibility()] [

    num = choose(true > false) ? "Yes" : "No"

    msg = choose(true != false) ? "True == False" : choose(true > false) ? "true is less than false" : "true is Greater Than False"

]