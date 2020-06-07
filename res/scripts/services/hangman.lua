local S = SERVICE_MODULE("hangman", "7308432")


function S.load(args)
end

S:state(SERVICE_STATE_CALL, {
    enter = function(self)
        print("Hangman: call started")
    end,
    tick = function(self)
        -- TODO: Implement Hangman
    end,
    exit = function(self)
        print("Hangman: ending call")
        sound.unload_bank("hangman")
    end
})


S:state(SERVICE_STATE_CALL_IN, {
    enter = function(self) 
        sound.load_bank("hangman")
    end,
    tick = function(self)
        service.wait(random_float(4.0, 8.0))
        service.accept_call()
    end
})

S:state(SERVICE_STATE_CALL_OUT, {
    enter = function(self)
        sound.load_bank("hangman")
    end,
    tick = function(self)
        service.wait(random_float(4.0, 8.0))
        service.accept_call()
    end
})

function S.unload(args)
end

return S