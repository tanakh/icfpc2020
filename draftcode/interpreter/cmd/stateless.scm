(load "./prelude.scm")
(define (galaxy) (delay (force ((force (delay (force ((force (c)) (delay (force ((force (delay (force ((force (b)) (b))))) (delay (force ((force (delay (force ((force (b)) (delay (force ((force (b)) (delay (force ((force (mycons)) 0)))))))))) (delay (force ((force (delay (force ((force (c)) (delay (force ((force (delay (force ((force (b)) (b))))) (mycons)))))))) (delay (force ((force (delay (force ((force (c)) (mycons))))) (nil))))))))))))))))) (delay (force ((force (delay (force ((force (c)) (delay (force ((force (delay (force ((force (b)) (mycons))))) (delay (force ((force (delay (force ((force (c)) (mycons))))) (nil))))))))))) (nil))))))))
(define (result) (delay (force ((force (delay (force ((force (galaxy)) (nil))))) (delay (force ((force (delay (force ((force (mycons)) 0)))) 0)))))))
(printseq (result))
