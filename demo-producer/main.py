import json
from time import sleep
from kafka import KafkaProducer

producer = KafkaProducer(bootstrap_servers='localhost:9092')

def delivery_report(err, msg):
    """ Called once for each message produced to indicate delivery result.
        Triggered by poll() or flush(). """
    if err is not None:
        print('Message delivery failed: {}'.format(err))
    else:
        print('Message delivered to {} [{}]'.format(msg.topic(), msg.partition()))


for i in range(2):

    data = {
        "id": i,
        "val": "test value" + str(i)
    }

    # Asynchronously produce a message. The delivery report callback will
    # be triggered from the call to poll() above, or flush() below, when the
    # message has been successfully delivered or failed permanently.
    producer.send('quickstart', json.dumps(data).encode('utf-8'))
    print("throw " + str(i))
    sleep(2)

